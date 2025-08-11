use crate::tools::code_executor::CodeLanguage;

pub fn get_dockerfile(language: &CodeLanguage) -> &'static str {
    match language {
        CodeLanguage::JavaScript => JS_DOCKERFILE,
        CodeLanguage::TypeScript => TS_DOCKERFILE,
        CodeLanguage::Python => PYTHON_DOCKERFILE,
        CodeLanguage::Rust => RUST_DOCKERFILE,
        CodeLanguage::Bash => unimplemented!(),
    }
}

pub fn get_dockerfile_info(language: &CodeLanguage) -> (&str, &str, &str) {
    let (base_image, file_name, cmd) = match language {
        CodeLanguage::JavaScript => ("node:20-alpine", "main.js", "node main.js"),
        CodeLanguage::TypeScript => ("node:20-alpine", "main.ts", "pnpm tsx main.ts"),
        CodeLanguage::Python => ("python:3.13-alpine", "main.py", "python main.py"),
        CodeLanguage::Rust => ("rust:1.85-slim", "main.rs", "./target/debug/temp"),
        CodeLanguage::Bash => ("alpine:latest", "script.sh", "sh script.sh"),
    };
    (base_image, file_name, cmd)
}

const JS_DOCKERFILE: &str = r#"
FROM node:20-alpine

ARG DEPENDENCIES
ENV PNPM_HOME="/tmp/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN npm install -g pnpm@9

USER 1000:1000
RUN mkdir -p /tmp/home /tmp/pnpm
WORKDIR /app

RUN pnpm init
RUN if [ -n "$DEPENDENCIES" ]; then pnpm install $DEPENDENCIES; fi

COPY main.js .

CMD ["node", "main.js"]
"#;

const TS_DOCKERFILE: &str = r#"
FROM node:20-alpine

ARG DEPENDENCIES
ENV PNPM_HOME="/tmp/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN npm install -g pnpm@9

USER 1000:1000
RUN mkdir -p /tmp/home /tmp/pnpm
WORKDIR /app

RUN pnpm init
RUN pnpm install tsx $DEPENDENCIES

COPY main.ts .

CMD ["pnpm", "tsx", "main.ts"]
"#;

const PYTHON_DOCKERFILE: &str = r#"
FROM python:3.13-alpine

ARG DEPENDENCIES
ENV PYTHONUSERBASE="/tmp/python"
ENV PATH="/tmp/python/bin:$PATH"
ENV PYTHONPATH="/tmp/python/lib/python3.13/site-packages:$PYTHONPATH"

USER 1000:1000
RUN mkdir -p /tmp/home /tmp/python
WORKDIR /app

RUN if [ -n "$DEPENDENCIES" ]; then pip install --user --no-cache-dir ${DEPENDENCIES}; fi

COPY main.py .

CMD ["python", "main.py"]
"#;

const RUST_DOCKERFILE: &str = r#"
FROM rust:1.85-slim

ARG DEPENDENCIES

USER 1000:1000
RUN mkdir -p /tmp/home
WORKDIR /app

RUN cargo init --name temp
RUN if [ -n "$DEPENDENCIES" ]; then cargo add $DEPENDENCIES; fi
RUN cargo build

COPY main.rs src/
RUN cargo build

CMD ["./target/debug/temp"]
"#;
