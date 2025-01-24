use super::{
    client_channel::*,
    log_prefixes,
    webrtc::{webrtc_action_with_timeout, Options},
};
use crate::gen::google;
use crate::gen::proto::rpc::v1::{
    auth_service_client::AuthServiceClient, AuthenticateRequest, Credentials,
};
use crate::gen::proto::rpc::webrtc::v1::{
    call_response::Stage, call_update_request::Update,
    signaling_service_client::SignalingServiceClient, CallUpdateRequest,
    OptionalWebRtcConfigRequest, OptionalWebRtcConfigResponse,
};
use crate::gen::proto::rpc::webrtc::v1::{
    CallRequest, IceCandidate, Metadata, RequestHeaders, Strings,
};
use crate::rpc::webrtc;
use ::http::header::HeaderName;
use ::http::{
    uri::{Authority, Parts, PathAndQuery, Scheme},
    HeaderValue, Version,
};
use ::viam_mdns::{discover, Response};
use ::webrtc::ice_transport::{
    ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
    ice_connection_state::RTCIceConnectionState,
};
use ::webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use anyhow::{Context, Result};
use core::fmt;
use futures::stream::FuturesUnordered;
use futures_util::{pin_mut, stream::StreamExt};
use local_ip_address::list_afinet_netifas;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
    task::{Context as TaskContext, Poll},
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, watch};
use tonic::codegen::BoxFuture;
use tonic::transport::{Body, Channel, Uri};
use tonic::{body::BoxBody, transport::ClientTlsConfig};
use tower::{Service, ServiceBuilder};
use tower_http::auth::AddAuthorization;
use tower_http::auth::AddAuthorizationLayer;
use tower_http::set_header::{SetRequestHeader, SetRequestHeaderLayer};

// gRPC status codes
const STATUS_CODE_OK: i32 = 0;
const STATUS_CODE_UNKNOWN: i32 = 2;
const STATUS_CODE_RESOURCE_EXHAUSTED: i32 = 8;

pub const VIAM_MDNS_SERVICE_NAME: &'static str = "_rpc._tcp.local";

type SecretType = String;

#[derive(Clone)]
/// A communication channel to a given uri. The channel is either a direct tonic channel,
/// or a webRTC channel.
pub enum ViamChannel {
    Direct(Channel),
    DirectPreAuthorized(AddAuthorization<SetRequestHeader<Channel, HeaderValue>>),
    WebRTC(Arc<WebRTCClientChannel>),
}

#[derive(Debug, Clone)]
pub struct RPCCredentials {
    entity: Option<String>,
    credentials: Credentials,
}

impl RPCCredentials {
    pub fn new(entity: Option<String>, r#type: SecretType, payload: String) -> Self {
        Self {
            credentials: Credentials { r#type, payload },
            entity,
        }
    }
}

impl ViamChannel {
    async fn create_resp(
        channel: &mut Arc<WebRTCClientChannel>,
        stream: crate::gen::proto::rpc::webrtc::v1::Stream,
        request: http::Request<BoxBody>,
        response: http::response::Builder,
    ) -> http::Response<Body> {
        let (parts, body) = request.into_parts();
        let mut status_code = STATUS_CODE_OK;
        let stream_id = stream.id;
        let metadata = Some(metadata_from_parts(&parts));
        let headers = RequestHeaders {
            method: parts
                .uri
                .path_and_query()
                .map(PathAndQuery::to_string)
                .unwrap_or_default(),
            metadata,
            timeout: None,
        };

        if let Err(e) = channel.write_headers(&stream, headers).await {
            log::error!("error writing headers: {e}");
            channel.close_stream_with_recv_error(stream_id, e);
            status_code = STATUS_CODE_UNKNOWN;
        }

        let data = hyper::body::to_bytes(body).await.unwrap().to_vec();
        if let Err(e) = channel.write_message(Some(stream), data).await {
            log::error!("error sending message: {e}");
            channel.close_stream_with_recv_error(stream_id, e);
            status_code = STATUS_CODE_UNKNOWN;
        };

        let body = match channel.resp_body_from_stream(stream_id) {
            Ok(body) => body,
            Err(e) => {
                log::error!("error receiving response from stream: {e}");
                channel.close_stream_with_recv_error(stream_id, e);
                status_code = STATUS_CODE_UNKNOWN;
                Body::empty()
            }
        };

        let response = if status_code != STATUS_CODE_OK {
            response.header("grpc-status", &status_code.to_string())
        } else {
            response
        };

        response.body(body).unwrap()
    }
}

impl Service<http::Request<BoxBody>> for ViamChannel {
    type Response = http::Response<Body>;
    type Error = tonic::transport::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        match self {
            Self::Direct(channel) => channel.poll_ready(cx),
            Self::DirectPreAuthorized(channel) => channel.poll_ready(cx),
            Self::WebRTC(_channel) => Poll::Ready(Ok(())),
        }
    }

    fn call(&mut self, request: http::Request<BoxBody>) -> Self::Future {
        match self {
            Self::Direct(channel) => Box::pin(channel.call(request)),
            Self::DirectPreAuthorized(channel) => Box::pin(channel.call(request)),
            Self::WebRTC(channel) => {
                let mut channel = channel.clone();
                let fut = async move {
                    let response = http::response::Response::builder()
                        // standardized gRPC headers.
                        .header("content-type", "application/grpc")
                        .version(Version::HTTP_2);

                    match channel.new_stream() {
                        Err(e) => {
                            log::error!("{e}");
                            let response = response
                                .header("grpc-status", &STATUS_CODE_RESOURCE_EXHAUSTED.to_string())
                                .body(Body::default())
                                .unwrap();

                            Ok(response)
                        }
                        Ok(stream) => {
                            Ok(Self::create_resp(&mut channel, stream, request, response).await)
                        }
                    }
                };
                Box::pin(fut)
            }
        }
    }
}

/// Options for modifying the connection parameters
#[derive(Debug)]
pub struct DialOptions {
    credentials: Option<RPCCredentials>,
    webrtc_options: Option<Options>,
    uri: Option<Parts>,
    disable_mdns: bool,
    allow_downgrade: bool,
    insecure: bool,
}
#[derive(Clone)]
pub struct WantsCredentials(());
#[derive(Clone)]
pub struct WantsUri(());
#[derive(Clone)]
pub struct WithCredentials(());
#[derive(Clone)]
pub struct WithoutCredentials(());

pub trait AuthMethod {}
impl AuthMethod for WithCredentials {}
impl AuthMethod for WithoutCredentials {}
/// A DialBuilder allows us to set options before establishing a connection to a server
#[allow(dead_code)]
pub struct DialBuilder<T> {
    state: T,
    config: DialOptions,
}

impl<T> fmt::Debug for DialBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dial")
            .field("State", &format_args!("{}", &std::any::type_name::<T>()))
            .field("Opt", &format_args!("{:?}", self.config))
            .finish()
    }
}

impl DialOptions {
    /// Creates a new DialBuilder
    pub fn builder() -> DialBuilder<WantsUri> {
        DialBuilder {
            state: WantsUri(()),
            config: DialOptions {
                credentials: None,
                uri: None,
                allow_downgrade: false,
                disable_mdns: false,
                insecure: false,
                webrtc_options: None,
            },
        }
    }
}

impl DialBuilder<WantsUri> {
    /// Sets the uri to connect to
    pub fn uri(self, uri: &str) -> DialBuilder<WantsCredentials> {
        let uri_parts = uri_parts_with_defaults(uri);
        DialBuilder {
            state: WantsCredentials(()),
            config: DialOptions {
                credentials: None,
                uri: Some(uri_parts),
                allow_downgrade: false,
                disable_mdns: false,
                insecure: false,
                webrtc_options: None,
            },
        }
    }
}
impl DialBuilder<WantsCredentials> {
    /// Tells connecting logic to not expect/require credentials
    pub fn without_credentials(self) -> DialBuilder<WithoutCredentials> {
        DialBuilder {
            state: WithoutCredentials(()),
            config: DialOptions {
                credentials: None,
                uri: self.config.uri,
                allow_downgrade: false,
                disable_mdns: false,
                insecure: false,
                webrtc_options: None,
            },
        }
    }
    /// Sets credentials to use when connecting
    pub fn with_credentials(self, creds: RPCCredentials) -> DialBuilder<WithCredentials> {
        DialBuilder {
            state: WithCredentials(()),
            config: DialOptions {
                credentials: Some(creds),
                uri: self.config.uri,
                allow_downgrade: false,
                disable_mdns: false,
                insecure: false,
                webrtc_options: None,
            },
        }
    }
}

impl<T: AuthMethod> DialBuilder<T> {
    /// Attempts to connect insecurely with scheme of HTTP as a default
    pub fn insecure(mut self) -> Self {
        self.config.insecure = true;
        self
    }
    /// Allows for downgrading and attempting to connect via HTTP if HTTPS fails
    pub fn allow_downgrade(mut self) -> Self {
        self.config.allow_downgrade = true;
        self
    }
    /// Disables connection via mDNS
    pub fn disable_mdns(mut self) -> Self {
        self.config.disable_mdns = true;
        self
    }

    /// Overrides any default connection behavior, forcing direct connection. Note that
    /// the connection itself will fail if it is between a client and server on separate
    /// networks and not over webRTC
    pub fn disable_webrtc(mut self) -> Self {
        let webrtc_options = Options::default().disable_webrtc();
        self.config.webrtc_options = Some(webrtc_options);
        self
    }

    async fn get_addr_from_interface(
        iface: (&str, Vec<&IpAddr>),
        candidates: &Vec<String>,
    ) -> Option<String> {
        let addresses: Vec<Ipv4Addr> = iface
            .1
            .iter()
            .filter_map(|ip| match ip {
                IpAddr::V4(v4) => Some(*v4),
                IpAddr::V6(_) => None,
            })
            .collect();

        let mut resp: Option<Response> = None;
        for ipv4 in addresses {
            for candidate in candidates {
                let discovery = discover::interface_with_loopback(
                    VIAM_MDNS_SERVICE_NAME,
                    Duration::from_millis(250),
                    ipv4,
                )
                .ok()?;
                let stream = discovery.listen();
                pin_mut!(stream);
                while let Some(Ok(response)) = stream.next().await {
                    if let Some(hostname) = response.hostname() {
                        // Machine uris come in local ("my-cool-robot.abcdefg.local.viam.cloud")
                        // and non-local ("my-cool-robot.abcdefg.viam.cloud") forms. Sometimes
                        // (namely with micro-rdk), our mdns query can only see one (the local) version.
                        // However, users are typically passing the non-local version. By splitting at
                        // "viam" and taking the only the first value, we can still search for
                        // candidates based on the actual "my-cool-robot" name without being opinionated
                        // on whether the candidate is locally named or not.
                        let local_agnostic_candidate = candidate.as_str().split("viam").next()?;
                        if hostname.contains(local_agnostic_candidate) {
                            resp = Some(response);
                            break;
                        }
                    }
                    if resp.is_some() {
                        break;
                    }
                }
            }
        }

        let resp = resp?;
        let mut has_grpc = false;
        let mut has_webrtc = false;
        for field in resp.txt_records() {
            has_grpc = has_grpc || field.contains("grpc");
            has_webrtc = has_webrtc || field.contains("webrtc");
        }

        let ip_addr = match resp.ip_addr() {
            Some(std::net::IpAddr::V4(ip_v4)) => Some(ip_v4),
            Some(std::net::IpAddr::V6(_)) | None => None,
        };

        if !(has_grpc || has_webrtc) || ip_addr.is_none() {
            return None;
        }
        let mut local_addr = ip_addr?.to_string();
        local_addr.push(':');
        local_addr.push_str(&resp.port()?.to_string());
        Some(local_addr)
    }

    fn duplicate_uri(&self) -> Option<Parts> {
        match &self.config.uri {
            None => None,
            Some(uri) => duplicate_uri(uri),
        }
    }

    async fn get_mdns_uri(&self) -> Option<Parts> {
        log::debug!("{}", log_prefixes::MDNS_QUERY_ATTEMPT);
        if self.config.disable_mdns {
            return None;
        }

        let mut uri = self.duplicate_uri()?;
        let candidate = uri.authority.clone()?.to_string();

        let candidates: Vec<String> = vec![candidate.replace('.', "-"), candidate];

        let ifaces = list_afinet_netifas().ok()?;

        let ifaces: HashMap<&str, Vec<&IpAddr>> =
            ifaces.iter().fold(HashMap::new(), |mut map, (k, v)| {
                map.entry(k).or_default().push(v);
                map
            });

        let mut iface_futures = FuturesUnordered::new();
        for iface in ifaces {
            iface_futures.push(Self::get_addr_from_interface(iface, &candidates));
        }

        let mut local_addr: Option<String> = None;
        while let Some(maybe_addr) = iface_futures.next().await {
            if maybe_addr.is_some() {
                local_addr = maybe_addr;
                break;
            }
        }
        let local_addr = match local_addr {
            None => {
                log::debug!("Unable to connect via mDNS");
                return None;
            }
            Some(addr) => {
                log::debug!("{}: {addr}", log_prefixes::MDNS_ADDRESS_FOUND);
                addr
            }
        };

        let auth = local_addr.parse::<Authority>().ok()?;
        uri.authority = Some(auth);
        uri.scheme = Some(Scheme::HTTP);

        Some(uri)
    }

    async fn create_channel(
        allow_downgrade: bool,
        domain: &str,
        uri: Uri,
        for_mdns: bool,
    ) -> Result<Channel> {
        let mut chan = Channel::builder(uri.clone());
        if for_mdns {
            let tls_config = ClientTlsConfig::new().domain_name(domain);
            chan = chan.tls_config(tls_config)?;
        }
        let chan = match chan
            .connect()
            .await
            .with_context(|| format!("Connecting to {:?}", uri.clone()))
        {
            Ok(c) => c,
            Err(e) => {
                if allow_downgrade {
                    let mut uri_parts = uri.clone().into_parts();
                    uri_parts.scheme = Some(Scheme::HTTP);
                    let uri = Uri::from_parts(uri_parts)?;
                    Channel::builder(uri).connect().await?
                } else {
                    return Err(anyhow::anyhow!(e));
                }
            }
        };
        Ok(chan)
    }
}

impl DialBuilder<WithoutCredentials> {
    fn clone(&self) -> Self {
        DialBuilder {
            state: WithoutCredentials(()),
            config: DialOptions {
                credentials: None,
                webrtc_options: self.config.webrtc_options.clone(),
                uri: self.duplicate_uri(),
                disable_mdns: self.config.disable_mdns,
                allow_downgrade: self.config.allow_downgrade,
                insecure: self.config.insecure,
            },
        }
    }

    /// attempts to establish a connection without credentials to the DialBuilder's given uri
    async fn connect_inner(
        self,
        mdns_uri: Option<Parts>,
        mut original_uri_parts: Parts,
    ) -> Result<ViamChannel> {
        let webrtc_options = self.config.webrtc_options;
        let disable_webrtc = match &webrtc_options {
            Some(options) => options.disable_webrtc,
            None => false,
        };
        if self.config.insecure {
            original_uri_parts.scheme = Some(Scheme::HTTP);
        }
        let original_uri = Uri::from_parts(original_uri_parts)?;
        let uri2 = original_uri.clone();
        let uri = infer_remote_uri_from_authority(original_uri);
        let domain = uri2.authority().to_owned().unwrap().as_str();

        let mdns_uri = mdns_uri.and_then(|p| Uri::from_parts(p).ok());
        let attempting_mdns = mdns_uri.is_some();
        if attempting_mdns {
            log::debug!("Attempting to connect via mDNS");
        } else {
            log::debug!("Attempting to connect");
        }

        let channel = match mdns_uri {
            Some(uri) => Self::create_channel(self.config.allow_downgrade, domain, uri, true).await,
            // not actually an error necessarily, but we want to ensure that a channel is still
            // created with the default uri
            None => Err(anyhow::anyhow!("")),
        };

        let channel = match channel {
            Ok(c) => {
                log::debug!("Connected via mDNS");
                c
            }
            Err(e) => {
                if attempting_mdns {
                    log::debug!(
                        "Unable to connect via mDNS; falling back to robot URI. Error: {e}"
                    );
                }
                Self::create_channel(self.config.allow_downgrade, domain, uri.clone(), false)
                    .await?
            }
        };
        // TODO (RSDK-517) make maybe_connect_via_webrtc take a more generic type so we don't
        // need to add these dummy layers.
        let intercepted_channel = ServiceBuilder::new()
            .layer(AddAuthorizationLayer::basic(
                "fake username",
                "fake password",
            ))
            .layer(SetRequestHeaderLayer::overriding(
                HeaderName::from_static("rpc-host"),
                HeaderValue::from_str(domain)?,
            ))
            .service(channel.clone());

        if disable_webrtc {
            log::debug!("{}", log_prefixes::DIALED_GRPC);
            Ok(ViamChannel::Direct(channel.clone()))
        } else {
            match maybe_connect_via_webrtc(uri, intercepted_channel.clone(), webrtc_options).await {
                Ok(webrtc_channel) => Ok(ViamChannel::WebRTC(webrtc_channel)),
                Err(e) => {
                    log::error!("error connecting via webrtc: {e}. Attempting to connect directly");
                    log::debug!("{}", log_prefixes::DIALED_GRPC);
                    Ok(ViamChannel::Direct(channel.clone()))
                }
            }
        }
    }

    async fn connect_mdns(self, original_uri: Parts) -> Result<ViamChannel> {
        let mdns_uri =
            webrtc::action_with_timeout(self.get_mdns_uri(), Duration::from_millis(1500))
                .await
                .ok()
                .flatten()
                .ok_or(anyhow::anyhow!(
                    "Unable to establish connection via mDNS; uri not found"
                ))?;

        self.connect_inner(Some(mdns_uri), original_uri).await
    }

    pub async fn connect(self) -> Result<ViamChannel> {
        log::debug!("{}", log_prefixes::DIAL_ATTEMPT);
        let original_uri = self.duplicate_uri().ok_or(anyhow::anyhow!(
            "Attempting to connect but there was no uri"
        ))?;
        let original_uri2 = duplicate_uri(&original_uri).ok_or(anyhow::anyhow!(
            "Attempting to connect but there was no uri"
        ))?;
        // We want to short circuit and return the first `Ok` result from our connection
        // attempts, which `tokio::select!` does great. Buuuuut, we don't want to
        // abandon the `Err` results, and we want to provide comprehensive logging for
        // debugging purposes. Hence the loop and pinning. The pinning lets us reference
        // the same future multiple times, while the loop lets us immediately return on the
        // first `Ok` result while still seeing and logging any error results.
        tokio::pin! {
            let with_mdns = self.clone().connect_mdns(original_uri);
            let without_mdns = self.connect_inner(None, original_uri2);
        }
        let mut with_mdns_err: Option<anyhow::Error> = None;
        let mut without_mdns_err: Option<anyhow::Error> = None;
        while with_mdns_err.is_none() || without_mdns_err.is_none() {
            tokio::select! {
                with_mdns = &mut with_mdns, if with_mdns_err.is_none() => {
                    match with_mdns {
                        Ok(chan) => return Ok(chan),
                        Err(e) => {
                            log::debug!("Error connecting with mdns: {e}");
                            with_mdns_err = Some(e);
                        }
                    }
                }
                without_mdns = &mut without_mdns, if without_mdns_err.is_none() => {
                    match without_mdns {
                        Ok(chan) => return Ok(chan),
                        Err(e) => {
                            log::debug!("Error connecting without mdns: {e}");
                            without_mdns_err = Some(e);
                        }
                    }
                }
            }
        }
        Err(anyhow::anyhow!(
            "Unable to connect with or without mdns.
                    with_mdns err: {with_mdns_err:?}
                    without_mdns err: {without_mdns_err:?}"
        ))
    }
}

async fn get_auth_token(
    channel: &mut Channel,
    creds: Credentials,
    entity: String,
) -> Result<String> {
    let mut auth_service = AuthServiceClient::new(channel);
    let req = AuthenticateRequest {
        entity,
        credentials: Some(creds),
    };

    let rsp = auth_service.authenticate(req).await?;
    Ok(rsp.into_inner().access_token)
}

impl DialBuilder<WithCredentials> {
    fn clone(&self) -> Self {
        DialBuilder {
            state: WithCredentials(()),
            config: DialOptions {
                credentials: self.config.credentials.clone(),
                webrtc_options: self.config.webrtc_options.clone(),
                uri: self.duplicate_uri(),
                disable_mdns: self.config.disable_mdns,
                allow_downgrade: self.config.allow_downgrade,
                insecure: self.config.insecure,
            },
        }
    }

    async fn connect_inner(
        self,
        mdns_uri: Option<Parts>,
        mut original_uri_parts: Parts,
    ) -> Result<ViamChannel> {
        let is_insecure = self.config.insecure;

        let webrtc_options = self.config.webrtc_options;
        let disable_webrtc = match &webrtc_options {
            Some(options) => options.disable_webrtc,
            None => false,
        };

        if is_insecure {
            original_uri_parts.scheme = Some(Scheme::HTTP);
        }

        let original_uri = Uri::from_parts(original_uri_parts)?;

        let domain = original_uri.authority().unwrap().to_string();
        let uri_for_auth = infer_remote_uri_from_authority(original_uri.clone());

        let mdns_uri = mdns_uri.and_then(|p| Uri::from_parts(p).ok());
        let attempting_mdns = mdns_uri.is_some();

        let allow_downgrade = self.config.allow_downgrade;
        if attempting_mdns {
            log::debug!("Attempting to connect via mDNS");
        } else {
            log::debug!("Attempting to connect");
        }
        let channel = match mdns_uri {
            Some(uri) => Self::create_channel(allow_downgrade, &domain, uri, true).await,
            // not actually an error necessarily, but we want to ensure that a channel is still
            // created with the default uri
            None => Err(anyhow::anyhow!("")),
        };
        let real_channel = match channel {
            Ok(c) => {
                log::debug!("Connected via mDNS");
                c
            }
            Err(e) => {
                if attempting_mdns {
                    log::debug!(
                        "Unable to connect via mDNS; falling back to robot URI. Error: {e}"
                    );
                }
                Self::create_channel(allow_downgrade, &domain, uri_for_auth, false).await?
            }
        };

        log::debug!("{}", log_prefixes::ACQUIRING_AUTH_TOKEN);
        let token = get_auth_token(
            &mut real_channel.clone(),
            self.config
                .credentials
                .as_ref()
                .unwrap()
                .credentials
                .clone(),
            self.config
                .credentials
                .unwrap()
                .entity
                .unwrap_or_else(|| domain.clone()),
        )
        .await?;
        log::debug!("{}", log_prefixes::ACQUIRED_AUTH_TOKEN);

        let channel = ServiceBuilder::new()
            .layer(AddAuthorizationLayer::bearer(&token))
            .layer(SetRequestHeaderLayer::overriding(
                HeaderName::from_static("rpc-host"),
                HeaderValue::from_str(domain.as_str())?,
            ))
            .service(real_channel);

        if disable_webrtc {
            log::debug!("Connected via gRPC");
            Ok(ViamChannel::DirectPreAuthorized(channel))
        } else {
            match maybe_connect_via_webrtc(original_uri, channel.clone(), webrtc_options).await {
                Ok(webrtc_channel) => Ok(ViamChannel::WebRTC(webrtc_channel)),
                Err(e) => {
                    log::error!(
                    "Unable to establish webrtc connection due to error: [{e}]. Attempting direct connection."
                );
                    log::debug!("Connected via gRPC");
                    Ok(ViamChannel::DirectPreAuthorized(channel))
                }
            }
        }
    }

    async fn connect_mdns(self, original_uri: Parts) -> Result<ViamChannel> {
        // NOTE(benjirewis): Use a duration of 1500ms for getting the mDNS URI. I've anecdotally
        // seen times as great as 922ms to fetch a non-loopback mDNS URI. With an
        // interface_with_loopback query interval of 250ms, 1500ms here should give us time for ~6
        // queries.
        let mdns_uri =
            webrtc::action_with_timeout(self.get_mdns_uri(), Duration::from_millis(1500))
                .await
                .ok()
                .flatten()
                .ok_or(anyhow::anyhow!(
                    "Unable to establish connection via mDNS; uri not found"
                ))?;

        self.connect_inner(Some(mdns_uri), original_uri).await
    }

    /// attempts to establish a connection with credentials to the DialBuilder's given uri
    pub async fn connect(self) -> Result<ViamChannel> {
        log::debug!("{}", log_prefixes::DIAL_ATTEMPT);
        let original_uri = self.duplicate_uri().ok_or(anyhow::anyhow!(
            "Attempting to connect but there was no uri"
        ))?;
        let original_uri2 = duplicate_uri(&original_uri).ok_or(anyhow::anyhow!(
            "Attempting to connect but there was no uri"
        ))?;

        // We want to short circuit and return the first `Ok` result from our connection
        // attempts, which `tokio::select!` does great. Buuuuut, we don't want to
        // abandon the `Err` results, and we want to provide comprehensive logging for
        // debugging purposes. Hence the loop and pinning. The pinning lets us reference
        // the same future multiple times, while the loop lets us immediately return on the
        // first `Ok` result while still seeing and logging any error results.
        tokio::pin! {
            let with_mdns = self.clone().connect_mdns(original_uri);
            let without_mdns = self.connect_inner(None, original_uri2);
        }
        let mut with_mdns_err: Option<anyhow::Error> = None;
        let mut without_mdns_err: Option<anyhow::Error> = None;
        while with_mdns_err.is_none() || without_mdns_err.is_none() {
            tokio::select! {
                with_mdns = &mut with_mdns, if with_mdns_err.is_none() => {
                    match with_mdns {
                        Ok(chan) => return Ok(chan),
                        Err(e) => {
                            log::debug!("Error connecting with mdns: {e}");
                            with_mdns_err = Some(e);
                        }
                    }
                }
                without_mdns = &mut without_mdns, if without_mdns_err.is_none() => {
                    match without_mdns {
                        Ok(chan) => return Ok(chan),
                        Err(e) => {
                            log::debug!("Error connecting without mdns: {e}");
                            without_mdns_err = Some(e);
                        }
                    }
                }
            }
        }
        Err(anyhow::anyhow!(
            "Unable to connect with or without mdns.
                    with_mdns err: {with_mdns_err:?}
                    without_mdns err: {without_mdns_err:?}"
        ))
    }
}

async fn send_done_or_error_update(
    update: CallUpdateRequest,
    channel: AddAuthorization<SetRequestHeader<Channel, HeaderValue>>,
) {
    let mut signaling_client = SignalingServiceClient::new(channel.clone());

    if let Err(e) = signaling_client
        .call_update(update)
        .await
        .map_err(anyhow::Error::from)
        .map(|_| ())
    {
        log::error!("Error sending done or error update: {e}")
    }
}

async fn send_error_once(
    sent_error: Arc<AtomicBool>,
    uuid: &String,
    err: &anyhow::Error,
    channel: AddAuthorization<SetRequestHeader<Channel, HeaderValue>>,
) {
    if sent_error.load(Ordering::Acquire) {
        return;
    }

    let err = google::rpc::Status {
        code: google::rpc::Code::Unknown.into(),
        message: err.to_string(),
        details: Vec::new(),
    };
    sent_error.store(true, Ordering::Release);
    let update_request = CallUpdateRequest {
        uuid: uuid.to_string(),
        update: Some(Update::Error(err)),
    };

    send_done_or_error_update(update_request, channel).await
}

async fn send_done_once(
    sent_done: Arc<AtomicBool>,
    uuid: &String,
    channel: AddAuthorization<SetRequestHeader<Channel, HeaderValue>>,
) {
    if sent_done.load(Ordering::Acquire) {
        return;
    }
    sent_done.store(true, Ordering::Release);
    let update_request = CallUpdateRequest {
        uuid: uuid.to_string(),
        update: Some(Update::Done(true)),
    };

    send_done_or_error_update(update_request, channel).await
}

#[derive(Default)]
struct CallerUpdateStats {
    count: u128,
    total_duration: Duration,
    max_duration: Duration,
}

impl fmt::Display for CallerUpdateStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let average_duration = &self.total_duration.as_millis() / &self.count;
        writeln!(
            f,
            "Caller update statistics: num_updates: {}, average_duration: {}ms, max_duration: {}ms",
            &self.count,
            average_duration,
            &self.max_duration.as_millis()
        )?;
        Ok(())
    }
}

async fn maybe_connect_via_webrtc(
    uri: Uri,
    channel: AddAuthorization<SetRequestHeader<Channel, HeaderValue>>,
    webrtc_options: Option<Options>,
) -> Result<Arc<WebRTCClientChannel>> {
    let webrtc_options = webrtc_options.unwrap_or_else(|| Options::infer_from_uri(uri.clone()));
    let mut signaling_client = SignalingServiceClient::new(channel.clone());
    let response = match signaling_client
        .optional_web_rtc_config(OptionalWebRtcConfigRequest::default())
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            if e.code() == tonic::Code::Unimplemented {
                tonic::Response::new(OptionalWebRtcConfigResponse::default())
            } else {
                return Err(anyhow::anyhow!(e));
            }
        }
    };

    let optional_config = response.into_inner().config;
    let config = webrtc::extend_webrtc_config(webrtc_options.config, optional_config);

    let (peer_connection, data_channel) =
        webrtc::new_peer_connection_for_client(config, webrtc_options.disable_trickle_ice).await?;

    let sent_done_or_error = Arc::new(AtomicBool::new(false));
    let uuid_lock = Arc::new(RwLock::new("".to_string()));
    let uuid_for_ice_gathering_thread = uuid_lock.clone();

    // Using an mpsc channel to report unrecoverable errors during Signaling, so we
    // don't have to wait until the timeout expires before giving up on this attempt.
    // The size of the channel is set to 1 since any error (or success) should terminate the function
    let (is_open_s, mut is_open_r) = mpsc::channel(1);
    let on_open_is_open = is_open_s.clone();

    data_channel.on_open(Box::new(move || {
        let _ = on_open_is_open.try_send(None); // ignore sending errors, either an error (or success) was already sent or the operation will succeed
        Box::pin(async move {})
    }));

    let exchange_done = Arc::new(AtomicBool::new(false));
    let (remote_description_set_s, remote_description_set_r) = watch::channel(None);
    let ice_done = Arc::new(tokio::sync::Notify::new());
    let ice_done2 = ice_done.clone();
    let caller_update_stats = Arc::new(Mutex::new(CallerUpdateStats::default()));

    if !webrtc_options.disable_trickle_ice {
        let offer = peer_connection.create_offer(None).await?;
        let channel2 = channel.clone();
        let uuid_lock2 = uuid_lock.clone();
        let sent_done_or_error2 = sent_done_or_error.clone();

        let exchange_done = exchange_done.clone();

        let on_local_ice_candidate_failure = is_open_s.clone();

        let caller_update_stats = caller_update_stats.clone();
        let caller_update_stats2 = caller_update_stats.clone();
        peer_connection.on_ice_connection_state_change(Box::new(
            move |state: RTCIceConnectionState| {
                let caller_update_stats = caller_update_stats.clone();
                Box::pin(async move {
                    if state == RTCIceConnectionState::Completed {
                        let caller_update_stats_inner = caller_update_stats.lock().unwrap();
                        log::debug!("{}", caller_update_stats_inner);
                    }
                })
            },
        ));
        peer_connection.on_ice_candidate(Box::new(
            move |ice_candidate: Option<RTCIceCandidate>| {
                if exchange_done.load(Ordering::Acquire) {
                    return Box::pin(async move {});
                }
                let channel = channel2.clone();
                let sent_done_or_error = sent_done_or_error2.clone();
                let ice_done = ice_done.clone();
                let uuid_lock = uuid_lock2.clone();
                let on_local_ice_candidate_failure = on_local_ice_candidate_failure.clone();
                let mut remote_description_set_r = remote_description_set_r.clone();
                let caller_update_stats = caller_update_stats2.clone();
                Box::pin(async move {
                    // If the value in the watch channel has not been set yet, we wait until it does.
                    // Afterwards Some(()) should be visible to all watcher and any watcher waiting  will
                    // return
                    if remote_description_set_r.borrow().is_none() {
                        match webrtc_action_with_timeout(remote_description_set_r.changed()).await {
                            Ok(Err(e)) => {
                                let _ = on_local_ice_candidate_failure.try_send(Some(Box::new(
                                    anyhow::anyhow!(
                                        "remote description watch channel is closed with error {e}"
                                    ),
                                )));
                            }
                            Err(_) => {
                                log::info!(
                                    "timed out on_ice_candidate; remote description was never set"
                                );
                                let _ = on_local_ice_candidate_failure.try_send(Some(Box::new(
                                    anyhow::anyhow!("timed out waiting for remote description"),
                                )));
                            }
                            _ => (),
                        }
                    }

                    let uuid = uuid_lock.read().unwrap().to_string();
                    // Note(ethan): for reasons that aren't entirely clear to me, parallel dialing
                    // occasionally causes us to not receive a signaling client response when
                    // trying to establish a connection. This results in noisy error messages that
                    // fortunately are harmless (this problem seems to only ever affect one branch
                    // of the parallel dial, so we still end up with a successful connection).
                    // By checking if the `uuid` is empty, we can tell if we're in such a case and
                    // exit out before it results in logging noisy error messages.
                    //
                    // It would be lovely to understand this problem better, but given that it's
                    // not actually causing performance failures it's probably not worth the effort
                    // at this time.
                    if uuid.is_empty() {
                        log::debug!(
                            "UUID never updated. This is likely because we never received a response \
                            from the signaling client. This happens occasionally with parallel dialing \
                            and isn't concerning provided connection still occurs."
                        );
                        return;
                    }
                    let mut signaling_client = SignalingServiceClient::new(channel.clone());
                    match ice_candidate {
                        Some(ice_candidate) => {
                            log::debug!("Gathered local candidate of {ice_candidate}");
                            if sent_done_or_error.load(Ordering::Acquire) {
                                return;
                            }
                            let proto_candidate = ice_candidate_to_proto(ice_candidate).await;
                            match proto_candidate {
                                Ok(proto_candidate) => {
                                    let update_request = CallUpdateRequest {
                                        uuid: uuid.clone(),
                                        update: Some(Update::Candidate(proto_candidate)),
                                    };
                                    let call_update_start = Instant::now();
                                    if let Err(e) = webrtc_action_with_timeout(
                                        signaling_client.call_update(update_request),
                                    )
                                    .await
                                    .and_then(|resp| resp.map_err(anyhow::Error::from))
                                    {
                                        log::error!("Error sending ice candidate: {e}");
                                        let _ = on_local_ice_candidate_failure.try_send(Some(
                                            Box::new(anyhow::anyhow!(
                                                "Error sending ice candidate: {e}"
                                            )),
                                        ));
                                    }
                                    let mut caller_update_stats_inner =
                                        caller_update_stats.lock().unwrap();
                                    caller_update_stats_inner.count += 1;
                                    let call_update_duration = call_update_start.elapsed();
                                    if call_update_duration > caller_update_stats_inner.max_duration
                                    {
                                        caller_update_stats_inner.max_duration =
                                            call_update_duration;
                                    }
                                    caller_update_stats_inner.total_duration +=
                                        call_update_duration;
                                }
                                Err(e) => log::error!("Error parsing ice candidate: {e}"),
                            }
                        }
                        None => {
                            // will only be executed once when gathering is finished
                            ice_done.notify_one();
                            send_done_once(sent_done_or_error, &uuid, channel.clone()).await;
                        }
                    }
                })
            },
        ));

        peer_connection.set_local_description(offer).await?;
    }

    let local_description = peer_connection.local_description().await.unwrap();

    // Local SD will be multi-line, so use two log messages to indicate start, SD and end.
    log::debug!(
        "{}\n{}",
        log_prefixes::START_LOCAL_SESSION_DESCRIPTION,
        local_description.sdp
    );
    log::debug!("{}", log_prefixes::END_LOCAL_SESSION_DESCRIPTION);

    let sdp = encode_sdp(local_description)?;
    let call_request = CallRequest {
        sdp,
        disable_trickle: webrtc_options.disable_trickle_ice,
    };

    let client_channel = WebRTCClientChannel::new(peer_connection, data_channel).await;
    let client_channel_for_ice_gathering_thread = Arc::downgrade(&client_channel);
    let mut signaling_client = SignalingServiceClient::new(channel.clone());
    let mut call_client = signaling_client.call(call_request).await?.into_inner();

    let channel2 = channel.clone();
    let sent_done_or_error2 = sent_done_or_error.clone();
    tokio::spawn(async move {
        let uuid = uuid_for_ice_gathering_thread;
        let client_channel = client_channel_for_ice_gathering_thread;
        let init_received = AtomicBool::new(false);
        let sent_done = sent_done_or_error2;

        loop {
            let response = match webrtc_action_with_timeout(call_client.message())
                .await
                .and_then(|resp| resp.map_err(anyhow::Error::from))
            {
                Ok(cr) => match cr {
                    Some(cr) => cr,
                    None => {
                        // want to delay sending done until we either are actually done, or
                        // we hit a timeout
                        let _ = webrtc_action_with_timeout(ice_done2.notified()).await;
                        let uuid = uuid.read().unwrap().to_string();
                        send_done_once(sent_done.clone(), &uuid, channel2.clone()).await;
                        break;
                    }
                },
                Err(e) => {
                    log::error!("Error processing call response: {e}");
                    let _ = is_open_s.try_send(Some(Box::new(e)));
                    break;
                }
            };

            match response.stage {
                Some(Stage::Init(init)) => {
                    if init_received.load(Ordering::Acquire) {
                        let uuid = uuid.read().unwrap().to_string();
                        let e = anyhow::anyhow!("Init received more than once");
                        send_error_once(sent_done.clone(), &uuid, &e, channel2.clone()).await;
                        let _ = is_open_s.try_send(Some(Box::new(e)));
                        break;
                    }
                    init_received.store(true, Ordering::Release);
                    {
                        let mut uuid_s = uuid.write().unwrap();
                        uuid_s.clone_from(&response.uuid);
                    }

                    let answer = match decode_sdp(init.sdp) {
                        Ok(a) => a,
                        Err(e) => {
                            send_error_once(
                                sent_done.clone(),
                                &response.uuid,
                                &e,
                                channel2.clone(),
                            )
                            .await;
                            let _ = is_open_s.try_send(Some(Box::new(e)));
                            break;
                        }
                    };
                    {
                        let cc = match client_channel.upgrade() {
                            Some(cc) => cc,
                            None => {
                                break;
                            }
                        };
                        if let Err(e) = cc
                            .base_channel
                            .peer_connection
                            .set_remote_description(answer)
                            .await
                        {
                            let e = anyhow::Error::from(e);
                            send_error_once(
                                sent_done.clone(),
                                &response.uuid,
                                &e,
                                channel2.clone(),
                            )
                            .await;
                            let _ = is_open_s.try_send(Some(Box::new(e)));
                            break;
                        }
                    }
                    let _ = remote_description_set_s.send_replace(Some(()));
                    if webrtc_options.disable_trickle_ice {
                        send_done_once(sent_done.clone(), &response.uuid, channel2.clone()).await;
                        break;
                    }
                }

                Some(Stage::Update(update)) => {
                    let uuid_s = uuid.read().unwrap().to_string();
                    if !init_received.load(Ordering::Acquire) {
                        let e = anyhow::anyhow!("Got update before init stage");
                        send_error_once(sent_done.clone(), &uuid_s, &e, channel2.clone()).await;
                        let _ = is_open_s.try_send(Some(Box::new(e)));
                        break;
                    }

                    if response.uuid != *uuid.read().unwrap() {
                        let e = anyhow::anyhow!(
                            "uuid mismatch: have {}, want {}",
                            response.uuid,
                            uuid_s,
                        );
                        send_error_once(sent_done.clone(), &uuid_s, &e, channel2.clone()).await;
                        let _ = is_open_s.try_send(Some(Box::new(e)));
                        break;
                    }
                    match ice_candidate_from_proto(update.candidate) {
                        Ok(candidate) => {
                            let client_channel = match client_channel.upgrade() {
                                Some(cc) => cc,
                                None => {
                                    break;
                                }
                            };
                            log::debug!("Received remote ICE candidate of {candidate:#?}");
                            if let Err(e) = client_channel
                                .base_channel
                                .peer_connection
                                .add_ice_candidate(candidate)
                                .await
                            {
                                let e = anyhow::Error::from(e);
                                send_error_once(sent_done.clone(), &uuid_s, &e, channel2.clone())
                                    .await;
                                let _ = is_open_s.try_send(Some(Box::new(e)));
                                break;
                            }
                        }
                        Err(e) => log::error!("Error parsing ice candidate: {e}"),
                    }
                }
                None => continue,
            }
        }
    });

    // TODO (GOUT-11): create separate authorization if external_auth_addr and/or creds.Type is `Some`

    // Delay returning the client channel until data channel is open, so we don't lose messages
    let is_open = webrtc_action_with_timeout(is_open_r.recv()).await;
    match is_open {
        Ok(is_open) => {
            if let Some(Some(e)) = is_open {
                return Err(anyhow::anyhow!("Couldn't connect to peer with error {e}"));
            }
        }
        Err(_) => {
            return Err(anyhow::anyhow!("Timed out opening data channel."));
        }
    }

    exchange_done.store(true, Ordering::Release);
    let uuid = uuid_lock.read().unwrap().to_string();
    send_done_once(sent_done_or_error, &uuid, channel.clone()).await;
    Ok(client_channel)
}

async fn ice_candidate_to_proto(ice_candidate: RTCIceCandidate) -> Result<IceCandidate> {
    let ice_candidate = ice_candidate.to_json()?;
    Ok(IceCandidate {
        candidate: ice_candidate.candidate,
        sdp_mid: ice_candidate.sdp_mid,
        sdpm_line_index: ice_candidate.sdp_mline_index.map(u32::from),
        username_fragment: ice_candidate.username_fragment,
    })
}

fn ice_candidate_from_proto(proto: Option<IceCandidate>) -> Result<RTCIceCandidateInit> {
    match proto {
        Some(proto) => {
            let proto_sdpm: usize = proto.sdpm_line_index().try_into()?;
            let sdp_mline_index: Option<u16> = proto_sdpm.try_into().ok();

            Ok(RTCIceCandidateInit {
                candidate: proto.candidate.clone(),
                sdp_mid: Some(proto.sdp_mid().to_string()),
                sdp_mline_index,
                username_fragment: Some(proto.username_fragment().to_string()),
            })
        }
        None => Err(anyhow::anyhow!("No ice candidate provided")),
    }
}

fn decode_sdp(sdp: String) -> Result<RTCSessionDescription> {
    let sdp = String::from_utf8(base64::decode(sdp)?)?;
    Ok(serde_json::from_str::<RTCSessionDescription>(&sdp)?)
}

fn encode_sdp(sdp: RTCSessionDescription) -> Result<String> {
    let sdp = serde_json::to_vec(&sdp)?;
    Ok(base64::encode(sdp))
}

fn infer_remote_uri_from_authority(uri: Uri) -> Uri {
    let authority = uri.authority().map(Authority::as_str).unwrap_or_default();
    let is_local_connection = authority.contains(".local.viam.cloud")
        || authority.contains("localhost")
        || authority.contains("0.0.0.0");

    if !is_local_connection {
        if let Some((new_uri, _)) = Options::infer_signaling_server_address(&uri) {
            return Uri::from_parts(uri_parts_with_defaults(&new_uri)).unwrap_or(uri);
        }
    }
    uri
}

fn duplicate_uri(parts: &Parts) -> Option<Parts> {
    let uri = Uri::builder()
        .authority(parts.authority.clone()?)
        .path_and_query(parts.path_and_query.clone()?)
        .scheme(parts.scheme.clone()?);
    Some(uri.build().ok()?.into_parts())
}

fn uri_parts_with_defaults(uri: &str) -> Parts {
    let mut uri_parts = uri.parse::<Uri>().unwrap().into_parts();
    uri_parts.scheme = Some(Scheme::HTTPS);
    uri_parts.path_and_query = Some(PathAndQuery::from_static(""));
    uri_parts
}

fn metadata_from_parts(parts: &http::request::Parts) -> Metadata {
    let mut md = HashMap::new();
    for (k, v) in parts.headers.iter() {
        let k = k.to_string();
        let v = Strings {
            values: vec![HeaderValue::to_str(v).unwrap().to_string()],
        };
        md.insert(k, v);
    }
    Metadata { md }
}
