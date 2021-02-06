use proc_macro2::TokenStream;
use prost_build::{Config, Method as ProstMethod, Service as ProstService};
use std::io;
use std::path::{Path, PathBuf};
use tonic_build::{client, server, Method, Service};

fn main() {
    configure()
        .build_server(true)
        .build_client(false)
        .compile(&["gorums.proto"], &["proto"])
        .unwrap();
}

const GORUMS_CODEC_PATH: &str = "crate::codec::GorumsCodec";

struct GorumsService<'a> {
    s: &'a ProstService,
    methods: Vec<GorumsMethod<'a>>,
}

impl<'a> GorumsService<'a> {
    fn new(s: &'a ProstService) -> Self {
        let methods = s
            .methods()
            .into_iter()
            .map(|m| GorumsMethod { m: m })
            .collect();
        return Self { s, methods };
    }
}

impl<'a> Service for GorumsService<'a> {
    const CODEC_PATH: &'static str = GORUMS_CODEC_PATH;

    type Method = GorumsMethod<'a>;
    type Comment = String;

    fn name(&self) -> &str {
        self.s.name()
    }

    fn package(&self) -> &str {
        self.s.package()
    }

    fn identifier(&self) -> &str {
        self.s.identifier()
    }

    fn comment(&self) -> &[Self::Comment] {
        self.s.comment()
    }

    fn methods(&self) -> &[Self::Method] {
        &self.methods[..]
    }
}

struct GorumsMethod<'a> {
    m: &'a ProstMethod,
}

impl Method for GorumsMethod<'_> {
    const CODEC_PATH: &'static str = GORUMS_CODEC_PATH;
    type Comment = String;

    fn name(&self) -> &str {
        self.m.name()
    }

    fn identifier(&self) -> &str {
        self.m.identifier()
    }

    fn client_streaming(&self) -> bool {
        self.m.client_streaming()
    }

    fn server_streaming(&self) -> bool {
        self.m.server_streaming()
    }

    fn comment(&self) -> &[Self::Comment] {
        self.m.comment()
    }

    fn request_response_name(&self, proto_path: &str) -> (TokenStream, TokenStream) {
        self.m.request_response_name(proto_path)
    }
}
/**
 * The code below is copied (with slight modifications) from tonic (https://github.com/hyperium/tonic)
 * The tonic code is covered by the following copyright and permission notice:
 *
 * Copyright (c) 2020 Lucio Franco
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

/// Configure `tonic-build` code generation.
///
/// Use [`compile_protos`] instead if you don't need to tweak anything.
pub fn configure() -> Builder {
    Builder {
        build_client: true,
        build_server: true,
        out_dir: None,
        extern_path: Vec::new(),
        field_attributes: Vec::new(),
        type_attributes: Vec::new(),
        proto_path: "super".to_string(),
        format: true,
    }
}

/// Simple `.proto` compiling. Use [`configure`] instead if you need more options.
///
/// The include directory will be the parent folder of the specified path.
/// The package name will be the filename without the extension.
pub fn compile_protos(proto: impl AsRef<Path>) -> io::Result<()> {
    let proto_path: &Path = proto.as_ref();

    // directory the main .proto file resides in
    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    self::configure().compile(&[proto_path], &[proto_dir])?;

    Ok(())
}

struct ServiceGenerator {
    builder: Builder,
    clients: TokenStream,
    servers: TokenStream,
}

impl ServiceGenerator {
    fn new(builder: Builder) -> Self {
        ServiceGenerator {
            builder,
            clients: TokenStream::default(),
            servers: TokenStream::default(),
        }
    }
}

impl prost_build::ServiceGenerator for ServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, _buf: &mut String) {
        if self.builder.build_server {
            let server = server::generate(&GorumsService::new(&service), &self.builder.proto_path);
            self.servers.extend(server);
        }

        if self.builder.build_client {
            let client = client::generate(&GorumsService::new(&service), &self.builder.proto_path);
            self.clients.extend(client);
        }
    }

    fn finalize(&mut self, buf: &mut String) {
        if self.builder.build_client && !self.clients.is_empty() {
            let clients = &self.clients;

            let client_service = quote::quote! {
                #clients
            };

            let code = format!("{}", client_service);
            buf.push_str(&code);

            self.clients = TokenStream::default();
        }

        if self.builder.build_server && !self.servers.is_empty() {
            let servers = &self.servers;

            let server_service = quote::quote! {
                #servers
            };

            let code = format!("{}", server_service);
            buf.push_str(&code);

            self.servers = TokenStream::default();
        }
    }
}
/// Service generator builder.
#[derive(Debug, Clone)]
pub struct Builder {
    pub(crate) build_client: bool,
    pub(crate) build_server: bool,
    pub(crate) extern_path: Vec<(String, String)>,
    pub(crate) field_attributes: Vec<(String, String)>,
    pub(crate) type_attributes: Vec<(String, String)>,
    pub(crate) proto_path: String,

    out_dir: Option<PathBuf>,
    format: bool,
}

impl Builder {
    /// Enable or disable gRPC client code generation.
    pub fn build_client(mut self, enable: bool) -> Self {
        self.build_client = enable;
        self
    }

    /// Enable or disable gRPC server code generation.
    pub fn build_server(mut self, enable: bool) -> Self {
        self.build_server = enable;
        self
    }

    /// Enable the output to be formated by rustfmt.
    pub fn format(mut self, run: bool) -> Self {
        self.format = run;
        self
    }

    /// Set the output directory to generate code to.
    ///
    /// Defaults to the `OUT_DIR` environment variable.
    pub fn out_dir(mut self, out_dir: impl AsRef<Path>) -> Self {
        self.out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    /// Declare externally provided Protobuf package or type.
    ///
    /// Passed directly to `prost_build::Config.extern_path`.
    /// Note that both the Protobuf path and the rust package paths should both be fully qualified.
    /// i.e. Protobuf paths should start with "." and rust paths should start with "::"
    pub fn extern_path(mut self, proto_path: impl AsRef<str>, rust_path: impl AsRef<str>) -> Self {
        self.extern_path.push((
            proto_path.as_ref().to_string(),
            rust_path.as_ref().to_string(),
        ));
        self
    }

    /// Add additional attribute to matched messages, enums, and one-offs.
    ///
    /// Passed directly to `prost_build::Config.field_attribute`.
    pub fn field_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.field_attributes
            .push((path.as_ref().to_string(), attribute.as_ref().to_string()));
        self
    }

    /// Add additional attribute to matched messages, enums, and one-offs.
    ///
    /// Passed directly to `prost_build::Config.type_attribute`.
    pub fn type_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.type_attributes
            .push((path.as_ref().to_string(), attribute.as_ref().to_string()));
        self
    }

    /// Set the path to where tonic will search for the Request/Response proto structs
    /// live relative to the module where you call `include_proto!`.
    ///
    /// This defaults to `super` since tonic will generate code in a module.
    pub fn proto_path(mut self, proto_path: impl AsRef<str>) -> Self {
        self.proto_path = proto_path.as_ref().to_string();
        self
    }

    /// Compile the .proto files and execute code generation.
    pub fn compile<P>(self, protos: &[P], includes: &[P]) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        self.compile_with_config(Config::new(), protos, includes)
    }

    /// Compile the .proto files and execute code generation using a
    /// custom `prost_build::Config`.
    pub fn compile_with_config<P>(
        self,
        mut config: Config,
        protos: &[P],
        includes: &[P],
    ) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let out_dir = if let Some(out_dir) = self.out_dir.as_ref() {
            out_dir.clone()
        } else {
            PathBuf::from(std::env::var("OUT_DIR").unwrap())
        };

        let format = self.format;

        config.out_dir(out_dir.clone());
        for (proto_path, rust_path) in self.extern_path.iter() {
            config.extern_path(proto_path, rust_path);
        }
        for (prost_path, attr) in self.field_attributes.iter() {
            config.field_attribute(prost_path, attr);
        }
        for (prost_path, attr) in self.type_attributes.iter() {
            config.type_attribute(prost_path, attr);
        }
        config.service_generator(Box::new(ServiceGenerator::new(self)));

        config.compile_protos(protos, includes)?;

        if format {
            tonic_build::fmt(out_dir.to_str().expect("Expected utf8 out_dir"));
        }

        Ok(())
    }
}
