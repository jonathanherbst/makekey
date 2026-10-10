FROM mcr.microsoft.com/devcontainers/base:noble

RUN apt update && apt install -y curl

USER vscode

RUN curl --proto '=https' --tlsv1.2 -LsSf https://sh.rustup.rs | bash -s -- -y