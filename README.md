# README

This service receives Tailscale webhooks on endpoint `/tailscale-webhook`

Supports all of the Opentelemetry environment variables

If you push traces to remote, use:
OTEL_EXPORTER_OTLP_ENDPOINT="<grpc_endpoint>"

Configuration example is provided in examples/config.toml
To override, use env vars prefixed with TAILFORWARD_:
TAILFORWARD_DEBUG=true
