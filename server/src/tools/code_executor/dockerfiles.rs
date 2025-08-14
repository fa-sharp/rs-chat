use const_format::formatcp;

use crate::tools::code_executor::CodeLanguage;

pub fn get_dockerfile(language: &CodeLanguage) -> &'static str {
    match language {
        CodeLanguage::JavaScript => JS_DOCKERFILE,
        CodeLanguage::TypeScript => TS_DOCKERFILE,
        CodeLanguage::Python => PYTHON_DOCKERFILE,
        CodeLanguage::Rust => RUST_DOCKERFILE,
        CodeLanguage::Go => GO_DOCKERFILE,
        CodeLanguage::Bash => BASH_DOCKERFILE,
    }
}

pub fn get_dockerfile_info(language: &CodeLanguage) -> (&str, &str, &str) {
    let (base_image, file_name, cmd) = match language {
        CodeLanguage::JavaScript => (JS_IMAGE, "main.js", "node main.js"),
        CodeLanguage::TypeScript => (JS_IMAGE, "main.ts", "pnpm tsx main.ts"),
        CodeLanguage::Python => (PYTHON_IMAGE, "main.py", "python main.py"),
        CodeLanguage::Rust => (RUST_IMAGE, "main.rs", "./target/debug/temp"),
        CodeLanguage::Go => (GO_IMAGE, "main.go", "go run main.go"),
        CodeLanguage::Bash => (BASH_IMAGE, "script.sh", "bash script.sh"),
    };
    (base_image, file_name, cmd)
}

const JS_IMAGE: &str = "node:20-slim";
const PYTHON_IMAGE: &str = "python:3.13-slim";
const RUST_IMAGE: &str = "rust:1.85-slim";
const GO_IMAGE: &str = "golang:1.24";
const BASH_IMAGE: &str = "bash:5.3";

const SET_USER_AND_HOME_DIR: &str = r#"
USER 1000:1000
RUN mkdir -p /tmp/home
WORKDIR /app
"#;

const JS_DOCKERFILE: &str = formatcp!(
    r#"
FROM {JS_IMAGE}

ARG DEPENDENCIES
ENV PNPM_HOME="/opt/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN mkdir -p /opt/pnpm && chown 1000:1000 /opt/pnpm
RUN npm install -g pnpm@9

{SET_USER_AND_HOME_DIR}

RUN pnpm init
RUN if [ -n "$DEPENDENCIES" ]; then pnpm install $DEPENDENCIES; fi

COPY main.js .

CMD ["node", "main.js"]
"#
);

const TS_DOCKERFILE: &str = formatcp!(
    r#"
FROM {JS_IMAGE}

ARG DEPENDENCIES
ENV PNPM_HOME="/opt/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN mkdir -p /opt/pnpm && chown 1000:1000 /opt/pnpm
RUN npm install -g pnpm@9

{SET_USER_AND_HOME_DIR}

RUN pnpm init
RUN pnpm install tsx $DEPENDENCIES

COPY main.ts .

CMD ["pnpm", "tsx", "main.ts"]
"#
);

const PYTHON_DOCKERFILE: &str = formatcp!(
    r#"
FROM {PYTHON_IMAGE}

ARG DEPENDENCIES
ENV PYTHONUSERBASE="/opt/python"
ENV PATH="/opt/python/bin:$PATH"

RUN mkdir -p /opt/python && chown 1000:1000 /opt/python

{SET_USER_AND_HOME_DIR}

RUN if [ -n "$DEPENDENCIES" ]; then pip install --user --no-cache-dir $DEPENDENCIES; fi

COPY main.py .

CMD ["python", "main.py"]
"#
);

const RUST_DOCKERFILE: &str = formatcp!(
    r#"
FROM {RUST_IMAGE}
RUN apt-get update -qq && apt-get install -y -qq pkg-config libssl-dev ca-certificates && apt-get clean

ARG DEPENDENCIES

{SET_USER_AND_HOME_DIR}

RUN cargo init --name temp
RUN if [ -n "$DEPENDENCIES" ]; then cargo add $DEPENDENCIES; fi
RUN cargo build

COPY --chown=1000:1000 main.rs src/
RUN touch src/main.rs
RUN cargo build

CMD ["./target/debug/temp"]
"#
);

const GO_DOCKERFILE: &str = formatcp!(
    r#"
FROM {GO_IMAGE}

ARG DEPENDENCIES

{SET_USER_AND_HOME_DIR}

RUN go mod init temp
RUN if [ -n "$DEPENDENCIES" ]; then go get $DEPENDENCIES; fi

COPY main.go .

CMD ["go", "run", "main.go"]
"#
);

const BASH_DOCKERFILE: &str = formatcp!(
    r#"
FROM {BASH_IMAGE}

ARG DEPENDENCIES

{SET_USER_AND_HOME_DIR}

RUN if [ -n "$DEPENDENCIES" ]; then apk add --no-cache $DEPENDENCIES; fi

COPY script.sh .

CMD ["bash", "script.sh"]
"#
);
