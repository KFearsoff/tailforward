# README

Supports all of the Opentelemetry environment variables

If you collect OTLP traces on localhost, use env var:
OTEL_SERVICE_NAME="tailforward"

If you push traces to remote, use:
OTEL_SERVICE_NAME="tailforward"
OTEL_EXPORTER_OTLP_ENDPOINT="<grpc_endpoint>"

Configuration example is provided in examples/config.toml
Also supports other formats that config.rs supports
To override, use env vars prefixed with TAILFORWARD_:
TAILFORWARD_DEBUG=1

You can also specify the config file to be used:
TAILFORWARD_CONFIG_FILE=/etc/tailforward.toml

Environment variables take higher precedence
