# Focus

Focus is a Samply component ran on the sites, which distributes tasks from [Beam.Proxy](https://github.com/samply/beam/) to the applications on the site and re-transmits the results through [Samply.Beam](https://github.com/samply/beam/). 

It is possible to specify [Blaze](https://github.com/samply/blaze) and SQL queries whose results are to be cached to speed up retrieval. The cached results expire after 24 hours. 

## Installation

### Samply/Bridgehead Integration

Focus is already included in the [Samply.Bridgehead deployment](https://github.com/samply/bridgehead/), a turnkey solution for deploying, maintaining, and monitoring applications in a medical IT environment.

### Standalone Installation

To run a standalone Focus, you need at least one running [Samply.Beam.Proxy](https://github.com/samply/beam/) and one running [Samply.Blaze FHIR store](https://github.com/samply/blaze).
You can compile and run this application via Cargo, however, we encourage the usage of the pre-compiled [docker images](https://hub.docker.com/r/samply/focus):

```bash
docker run --rm -e BEAM_PROXY_URL=http://localhost:8081 -e ENDPOINT_URL=http://localhost:8089/fhir/ -e PROXY_ID=proxy1.broker -e API_KEY=App1Secret -e BEAM_APP_ID_LONG=app1.broker.example.com samply/focus:latest
```

## Configuration

The following environment variables are mandatory for the usage of Focus. If compiling and running Focus yourself, they are provided as command line options as well. See `focus  --help` for details.

```bash
BEAM_PROXY_URL = "http://localhost:8081" 
ENDPOINT_URL = "http://localhost:8089/fhir/"
PROXY_ID = "proxy1.broker"
API_KEY = "App1Secret"
BEAM_APP_ID_LONG = "app1.broker.example.com"
```

### Optional variables

```bash
RETRY_COUNT = "32" # The maximum number of retries for beam and blaze healthchecks; default value: 32
ENDPOINT_TYPE = "blaze" # Type of the endpoint, allowed values: "blaze", "omop", "sql", "blaze-and-sql", "eucaim-api"; default value: "blaze"
EXPORTER_URL = " https://exporter.site/"  # The exporter URL
OBFUSCATE = "yes" # Should the results be obfuscated - the "master switch", allowed values: "yes", "no"; default value: "yes"
OBFUSCATE_BELOW_10_MODE = "1" # The mode of obfuscating values below 10: 0 - return zero, 1 - return ten, 2 - obfuscate using Laplace distribution and rounding, has no effect if OBFUSCATE = "no"; default value: 1
DELTA_PATIENT = "1." # Sensitivity parameter for obfuscating the counts in the Patient stratifier, has no effect if OBFUSCATE = "no"; default value: 1
DELTA_SPECIMEN = "20." # Sensitivity parameter for obfuscating the counts in the Specimen stratifier, has no effect if OBFUSCATE = "no"; default value: 20
DELTA_DIAGNOSIS = "3." # Sensitivity parameter for obfuscating the counts in the Diagnosis stratifier, has no effect if OBFUSCATE = "no"; default value: 3
DELTA_PROCEDURES = "1.7" # Sensitivity parameter for obfuscating the counts in the Procedures stratifier, has no effect if OBFUSCATE = "no"; default value: 1.7
DELTA_MEDICATION_STATEMENTS = "2.1" # Sensitivity parameter for obfuscating the counts in the Medication Statements stratifier, has no effect if OBFUSCATE = "no"; default value: 2.1
DELTA_HISTO = "20." # Sensitivity parameter for obfuscating the counts in the Histo stratifier, has no effect if OBFUSCATE = "no"; default value: 20
EPSILON = "0.1" # Privacy budget parameter for obfuscating the counts in the stratifiers, has no effect if OBFUSCATE = "no"; default value: 0.1
ROUNDING_STEP = "10" # The granularity of the rounding of the obfuscated values, has no effect if OBFUSCATE = "no"; default value: 10
PROJECTS_NO_OBFUSCATION = "exliquid;dktk_supervisors;exporter;ehds2" # Projects for which the results are not to be obfuscated, separated by ";" ; default value: "exliquid;dktk_supervisors;exporter;ehds2"
QUERIES_TO_CACHE = "queries_to_cache.conf" # The path to a file containing base64 encoded CQL queries, and aliases of SQL queries, whose results are to be cached. If not set, no results are cached
PROVIDER = "name" #EUCAIM provider name
PROVIDER_ICON = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEUAAACnej3aAAAAAXRSTlMAQObYZgAAAApJREFUCNdjYAAAAAIAAeIhvDMAAAAASUVORK5CYII=" # Base64 encoded EUCAIM provider icon in PNG format
AUTH_HEADER = "[Auth Type] XXXX" #Authorization header for accessing the store; Auth Type e.g. ApiKey, Basic, ...
EXPORTER_API_KEY = "XXXX" # Value of header x-api-key for accessing the Exporter application
```

In order to use Postgres querying, a Docker image built with the feature "dktk" needs to be used and this optional variable set:
```bash
POSTGRES_CONNECTION_STRING = "postgresql://postgres:Test.123@localhost:5432/postgres" # Postgres connection string
```

Additionally when using Postgres this optional variable can be set:
```bash
MAX_DB_ATTEMPTS = "8" # Max number of attempts to connect to the database; default value: 8
```

Obfuscating zero counts is by default switched off. To enable obfuscating zero counts, set the env. variable `OBFUSCATE_ZERO`. 

Optionally, you can provide the `TLS_CA_CERTIFICATES_DIR` environment variable to add additional trusted certificates, e.g., if you have a TLS-terminating proxy server in place. The application respects the `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`, and their respective lowercase equivalents.

Log level can be set using the `RUST_LOG` environment variable.

## Usage

Creating a sample focus healthcheck task using curl (body can be any string and is ignored):

```bash
curl -v -X POST -H "Content-Type: application/json" --data '{"id":"7fffefff-ffef-fcff-feef-feffffffffff","from":"app1.proxy1.broker","to":["app1.proxy1.broker"],"ttl":"10s","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"metadata":{"project":"focus-healthcheck"},"body":"wie geht es"}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks
```

Creating a sample task containing a [Blaze](https://github.com/samply/blaze) query using curl:

```bash
curl -v -X POST -H "Content-Type: application/json" --data '{"id":"7fffefff-ffef-fcff-feef-fefbffffeeff","from":"app1.proxy1.broker","to":["app1.proxy1.broker"],"ttl":"10s","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"metadata":{"project":"exliquid"},"body":"ewoJImxhbmciOiAiY3FsIiwKCSJsaWIiOiB7CgkJImNvbnRlbnQiOiBbCgkJCXsKCQkJCSJjb250ZW50VHlwZSI6ICJ0ZXh0L2NxbCIsCgkJCQkiZGF0YSI6ICJiR2xpY21GeWVTQlNaWFJ5YVdWMlpRcDFjMmx1WnlCR1NFbFNJSFpsY25OcGIyNGdKelF1TUM0d0p3cHBibU5zZFdSbElFWklTVkpJWld4d1pYSnpJSFpsY25OcGIyNGdKelF1TUM0d0p3b0tZMjlrWlhONWMzUmxiU0JzYjJsdVl6b2dKMmgwZEhBNkx5OXNiMmx1WXk1dmNtY25DbU52WkdWemVYTjBaVzBnYVdOa01UQTZJQ2RvZEhSd09pOHZhR3czTG05eVp5OW1hR2x5TDNOcFpDOXBZMlF0TVRBbkNtTnZaR1Z6ZVhOMFpXMGdVMkZ0Y0d4bFRXRjBaWEpwWVd4VWVYQmxPaUFuYUhSMGNITTZMeTltYUdseUxtSmliWEpwTG1SbEwwTnZaR1ZUZVhOMFpXMHZVMkZ0Y0d4bFRXRjBaWEpwWVd4VWVYQmxKd29LQ21OdmJuUmxlSFFnVUdGMGFXVnVkQW9LUWtKTlVrbGZVMVJTUVZSZlIwVk9SRVZTWDFOVVVrRlVTVVpKUlZJS0NrSkNUVkpKWDFOVVVrRlVYMFJGUmw5VFVFVkRTVTFGVGdwcFppQkpia2x1YVhScFlXeFFiM0IxYkdGMGFXOXVJSFJvWlc0Z1cxTndaV05wYldWdVhTQmxiSE5sSUh0OUlHRnpJRXhwYzNROFUzQmxZMmx0Wlc0K0NncENRazFTU1Y5VFZGSkJWRjlUUVUxUVRFVmZWRmxRUlY5VFZGSkJWRWxHU1VWU0NncENRazFTU1Y5VFZGSkJWRjlEVlZOVVQwUkpRVTVmVTFSU1FWUkpSa2xGVWdvS1FrSk5Va2xmVTFSU1FWUmZSRWxCUjA1UFUwbFRYMU5VVWtGVVNVWkpSVklLQ2tKQ1RWSkpYMU5VVWtGVVgwRkhSVjlUVkZKQlZFbEdTVVZTQ2dwQ1FrMVNTVjlUVkZKQlZGOUVSVVpmU1U1ZlNVNUpWRWxCVEY5UVQxQlZURUZVU1U5T0NuUnlkV1U9IgoJCQl9CgkJXSwKCQkicmVzb3VyY2VUeXBlIjogIkxpYnJhcnkiLAoJCSJzdGF0dXMiOiAiYWN0aXZlIiwKCQkidHlwZSI6IHsKCQkJImNvZGluZyI6IFsKCQkJCXsKCQkJCQkiY29kZSI6ICJsb2dpYy1saWJyYXJ5IiwKCQkJCQkic3lzdGVtIjogImh0dHA6Ly90ZXJtaW5vbG9neS5obDcub3JnL0NvZGVTeXN0ZW0vbGlicmFyeS10eXBlIgoJCQkJfQoJCQldCgkJfSwKCQkidXJsIjogInVybjp1dWlkOjdmZjUzMmFkLTY5ZTQtNDhlZC1hMmQzLTllZmFmYjYwOWY2MiIKCX0sCgkibWVhc3VyZSI6IHsKCQkiZ3JvdXAiOiBbCgkJCXsKCQkJCSJjb2RlIjogewoJCQkJCSJ0ZXh0IjogInBhdGllbnRzIgoJCQkJfSwKCQkJCSJwb3B1bGF0aW9uIjogWwoJCQkJCXsKCQkJCQkJImNvZGUiOiB7CgkJCQkJCQkiY29kaW5nIjogWwoJCQkJCQkJCXsKCQkJCQkJCQkJImNvZGUiOiAiaW5pdGlhbC1wb3B1bGF0aW9uIiwKCQkJCQkJCQkJInN5c3RlbSI6ICJodHRwOi8vdGVybWlub2xvZ3kuaGw3Lm9yZy9Db2RlU3lzdGVtL21lYXN1cmUtcG9wdWxhdGlvbiIKCQkJCQkJCQl9CgkJCQkJCQldCgkJCQkJCX0sCgkJCQkJCSJjcml0ZXJpYSI6IHsKCQkJCQkJCSJleHByZXNzaW9uIjogIkluSW5pdGlhbFBvcHVsYXRpb24iLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsLWlkZW50aWZpZXIiCgkJCQkJCX0KCQkJCQl9CgkJCQldLAoJCQkJInN0cmF0aWZpZXIiOiBbCgkJCQkJewoJCQkJCQkiY29kZSI6IHsKCQkJCQkJCSJ0ZXh0IjogIkdlbmRlciIKCQkJCQkJfSwKCQkJCQkJImNyaXRlcmlhIjogewoJCQkJCQkJImV4cHJlc3Npb24iOiAiR2VuZGVyIiwKCQkJCQkJCSJsYW5ndWFnZSI6ICJ0ZXh0L2NxbCIKCQkJCQkJfQoJCQkJCX0sCgkJCQkJewoJCQkJCQkiY29kZSI6IHsKCQkJCQkJCSJ0ZXh0IjogIkFnZSIKCQkJCQkJfSwKCQkJCQkJImNyaXRlcmlhIjogewoJCQkJCQkJImV4cHJlc3Npb24iOiAiQWdlQ2xhc3MiLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsIgoJCQkJCQl9CgkJCQkJfSwKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJInRleHQiOiAiQ3VzdG9kaWFuIgoJCQkJCQl9LAoJCQkJCQkiY3JpdGVyaWEiOiB7CgkJCQkJCQkiZXhwcmVzc2lvbiI6ICJDdXN0b2RpYW4iLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsIgoJCQkJCQl9CgkJCQkJfQoJCQkJXQoJCQl9LAoJCQl7CgkJCQkiY29kZSI6IHsKCQkJCQkidGV4dCI6ICJkaWFnbm9zaXMiCgkJCQl9LAoJCQkJImV4dGVuc2lvbiI6IFsKCQkJCQl7CgkJCQkJCSJ1cmwiOiAiaHR0cDovL2hsNy5vcmcvZmhpci91cy9jcWZtZWFzdXJlcy9TdHJ1Y3R1cmVEZWZpbml0aW9uL2NxZm0tcG9wdWxhdGlvbkJhc2lzIiwKCQkJCQkJInZhbHVlQ29kZSI6ICJDb25kaXRpb24iCgkJCQkJfQoJCQkJXSwKCQkJCSJwb3B1bGF0aW9uIjogWwoJCQkJCXsKCQkJCQkJImNvZGUiOiB7CgkJCQkJCQkiY29kaW5nIjogWwoJCQkJCQkJCXsKCQkJCQkJCQkJImNvZGUiOiAiaW5pdGlhbC1wb3B1bGF0aW9uIiwKCQkJCQkJCQkJInN5c3RlbSI6ICJodHRwOi8vdGVybWlub2xvZ3kuaGw3Lm9yZy9Db2RlU3lzdGVtL21lYXN1cmUtcG9wdWxhdGlvbiIKCQkJCQkJCQl9CgkJCQkJCQldCgkJCQkJCX0sCgkJCQkJCSJjcml0ZXJpYSI6IHsKCQkJCQkJCSJleHByZXNzaW9uIjogIkRpYWdub3NpcyIsCgkJCQkJCQkibGFuZ3VhZ2UiOiAidGV4dC9jcWwtaWRlbnRpZmllciIKCQkJCQkJfQoJCQkJCX0KCQkJCV0sCgkJCQkic3RyYXRpZmllciI6IFsKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJInRleHQiOiAiZGlhZ25vc2lzIgoJCQkJCQl9LAoJCQkJCQkiY3JpdGVyaWEiOiB7CgkJCQkJCQkiZXhwcmVzc2lvbiI6ICJEaWFnbm9zaXNDb2RlIiwKCQkJCQkJCSJsYW5ndWFnZSI6ICJ0ZXh0L2NxbC1pZGVudGlmaWVyIgoJCQkJCQl9CgkJCQkJfQoJCQkJXQoJCQl9LAoJCQl7CgkJCQkiY29kZSI6IHsKCQkJCQkidGV4dCI6ICJzcGVjaW1lbiIKCQkJCX0sCgkJCQkiZXh0ZW5zaW9uIjogWwoJCQkJCXsKCQkJCQkJInVybCI6ICJodHRwOi8vaGw3Lm9yZy9maGlyL3VzL2NxZm1lYXN1cmVzL1N0cnVjdHVyZURlZmluaXRpb24vY3FmbS1wb3B1bGF0aW9uQmFzaXMiLAoJCQkJCQkidmFsdWVDb2RlIjogIlNwZWNpbWVuIgoJCQkJCX0KCQkJCV0sCgkJCQkicG9wdWxhdGlvbiI6IFsKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJImNvZGluZyI6IFsKCQkJCQkJCQl7CgkJCQkJCQkJCSJjb2RlIjogImluaXRpYWwtcG9wdWxhdGlvbiIsCgkJCQkJCQkJCSJzeXN0ZW0iOiAiaHR0cDovL3Rlcm1pbm9sb2d5LmhsNy5vcmcvQ29kZVN5c3RlbS9tZWFzdXJlLXBvcHVsYXRpb24iCgkJCQkJCQkJfQoJCQkJCQkJXQoJCQkJCQl9LAoJCQkJCQkiY3JpdGVyaWEiOiB7CgkJCQkJCQkiZXhwcmVzc2lvbiI6ICJTcGVjaW1lbiIsCgkJCQkJCQkibGFuZ3VhZ2UiOiAidGV4dC9jcWwtaWRlbnRpZmllciIKCQkJCQkJfQoJCQkJCX0KCQkJCV0sCgkJCQkic3RyYXRpZmllciI6IFsKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJInRleHQiOiAic2FtcGxlX2tpbmQiCgkJCQkJCX0sCgkJCQkJCSJjcml0ZXJpYSI6IHsKCQkJCQkJCSJleHByZXNzaW9uIjogIlNhbXBsZVR5cGUiLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsIgoJCQkJCQl9CgkJCQkJfQoJCQkJXQoJCQl9CgkJXSwKCQkibGlicmFyeSI6ICJ1cm46dXVpZDo3ZmY1MzJhZC02OWU0LTQ4ZWQtYTJkMy05ZWZhZmI2MDlmNjIiLAoJCSJyZXNvdXJjZVR5cGUiOiAiTWVhc3VyZSIsCgkJInNjb3JpbmciOiB7CgkJCSJjb2RpbmciOiBbCgkJCQl7CgkJCQkJImNvZGUiOiAiY29ob3J0IiwKCQkJCQkic3lzdGVtIjogImh0dHA6Ly90ZXJtaW5vbG9neS5obDcub3JnL0NvZGVTeXN0ZW0vbWVhc3VyZS1zY29yaW5nIgoJCQkJfQoJCQldCgkJfSwKCQkic3RhdHVzIjogImFjdGl2ZSIsCgkJInN1YmplY3RDb2RlYWJsZUNvbmNlcHQiOiB7CgkJCSJjb2RpbmciOiBbCgkJCQl7CgkJCQkJImNvZGUiOiAiUGF0aWVudCIsCgkJCQkJInN5c3RlbSI6ICJodHRwOi8vaGw3Lm9yZy9maGlyL3Jlc291cmNlLXR5cGVzIgoJCQkJfQoJCQldCgkJfSwKCQkidXJsIjogInVybjp1dWlkOjVlZThkZTczLTM0N2UtNDdjYS1hMDE0LWYyZTcxNzY3YWRmYyIKCX0KfQ=="}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks
```

Creating a sample task containing an abstract syntax tree (AST) query using curl:

```bash
curl -v -X POST -H "Content-Type: application/json" --data '{"id":"7fffefff-ffef-fcff-feef-feffffffffff","from":"app1.proxy1.broker","to":["app1.proxy1.broker"],"ttl":"10s","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"metadata":{"project":"bbmri"},"body":"eyJsYW5nIjoiYXN0IiwicGF5bG9hZCI6ImV5SmhjM1FpT25zaWIzQmxjbUZ1WkNJNklrOVNJaXdpWTJocGJHUnlaVzRpT2x0N0ltOXdaWEpoYm1RaU9pSkJUa1FpTENKamFHbHNaSEpsYmlJNlczc2liM0JsY21GdVpDSTZJazlTSWl3aVkyaHBiR1J5Wlc0aU9sdDdJbXRsZVNJNkltZGxibVJsY2lJc0luUjVjR1VpT2lKRlVWVkJURk1pTENKemVYTjBaVzBpT2lJaUxDSjJZV3gxWlNJNkltMWhiR1VpZlN4N0ltdGxlU0k2SW1kbGJtUmxjaUlzSW5SNWNHVWlPaUpGVVZWQlRGTWlMQ0p6ZVhOMFpXMGlPaUlpTENKMllXeDFaU0k2SW1abGJXRnNaU0o5WFgxZGZWMTlMQ0pwWkNJNkltRTJaakZqWTJZekxXVmlaakV0TkRJMFppMDVaRFk1TFRSbE5XUXhNelZtTWpNME1DSjkifQ=="}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks
```

Creating a sample SQL task for a `SELECT_TEST` query using curl:
```bash
 curl -v -X POST -H "Content-Type: application/json" --data '{"id":"7fffefff-ffef-fcff-feef-feffffffffff","from":"app1.proxy1.broker","to":["app1.proxy1.broker"],"ttl":"10s","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"metadata":{"project":"exliquid"},"body":"eyJwYXlsb2FkIjoiU0VMRUNUX1RFU1QifQ=="}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks
 ```

 Creating a sample EUCAIM API query using curl:
```bash
 curl -v -X POST -H "Content-Type: application/json" --data '{"id":"7fffefff-ffef-fcff-feef-feffffffffff","from":"app1.proxy1.broker","to":["app1.proxy1.broker"],"ttl":"10s","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"metadata":{"project":"eucaim"},"body":"eyJsYW5nIjoiYXN0IiwicXVlcnkiOiJleUpoYzNRaU9uc2liM0JsY21GdVpDSTZJazlTSWl3aVkyaHBiR1J5Wlc0aU9sdDdJbTl3WlhKaGJtUWlPaUpCVGtRaUxDSmphR2xzWkhKbGJpSTZXM3NpYjNCbGNtRnVaQ0k2SWs5U0lpd2lZMmhwYkdSeVpXNGlPbHQ3SW10bGVTSTZJbE5PVDAxRlJFTlVNall6TkRrMU1EQXdJaXdpZEhsd1pTSTZJa1ZSVlVGTVV5SXNJbk41YzNSbGJTSTZJaUlzSW5aaGJIVmxJam9pVTA1UFRVVkVRMVF5TkRneE5UTXdNRGNpZlYxOUxIc2liM0JsY21GdVpDSTZJazlTSWl3aVkyaHBiR1J5Wlc0aU9sdDdJbXRsZVNJNklsTk9UMDFGUkVOVU5ETTVOREF4TURBeElpd2lkSGx3WlNJNklrVlJWVUZNVXlJc0luTjVjM1JsYlNJNkluVnlianB6Ym05dFpXUXRiM0puTDNOamRDSXNJblpoYkhWbElqb2lVMDVQVFVWRVExUXpOak0wTURZd01EVWlmVjE5TEhzaWIzQmxjbUZ1WkNJNklrOVNJaXdpWTJocGJHUnlaVzRpT2x0N0ltdGxlU0k2SWxKSlJERXdNekV4SWl3aWRIbHdaU0k2SWtWUlZVRk1VeUlzSW5ONWMzUmxiU0k2SW5WeWJqcHZhV1E2TWk0eE5pNDROREF1TVM0eE1UTTRPRE11Tmk0eU5UWWlMQ0oyWVd4MVpTSTZJbEpKUkRFd016RXlJbjFkZlN4N0ltOXdaWEpoYm1RaU9pSlBVaUlzSW1Ob2FXeGtjbVZ1SWpwYmV5SnJaWGtpT2lKVFRrOU5SVVJEVkRFeU16QXpOekF3TkNJc0luUjVjR1VpT2lKRlVWVkJURk1pTENKemVYTjBaVzBpT2lKMWNtNDZjMjV2YldWa0xXOXlaeTl6WTNRaUxDSjJZV3gxWlNJNklsTk9UMDFGUkVOVU56RTROVFF3TURFaWZWMTlMSHNpYjNCbGNtRnVaQ0k2SWs5U0lpd2lZMmhwYkdSeVpXNGlPbHQ3SW10bGVTSTZJa015TlRNNU1pSXNJblI1Y0dVaU9pSkZVVlZCVEZNaUxDSnplWE4wWlcwaU9pSm9kSFJ3T2k4dlltbHZiMjUwYjJ4dloza3ViM0puTDNCeWIycGxZM1J6TDI5dWRHOXNiMmRwWlhNdlltbHlibXhsZUNJc0luWmhiSFZsSWpvaVF6SXdNREUwTUNKOVhYMWRmVjE5TENKcFpDSTZJbUV5WkRrNU1qZGxMV1prTVRVdE5EY3hZUzFoWW1ReUxXSXhZMlk0TTJVM01XVXdNRjlmYzJWaGNtTm9YMTloTW1RNU9USTNaUzFtWkRFMUxUUTNNV0V0WVdKa01pMWlNV05tT0RObE56RmxNREFpZlE9PSJ9"}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks
```

Creating a sample [Exporter](https://github.com/samply/exporter) "execute" task containing an Exporter query using curl:

```bash
curl -v -X POST -H "Content-Type: application/json" --data '{"body":"ew0KICAicXVlcnktY29udGV4dCIgOiAiVUZKUFNrVkRWQzFKUkQxa01qaGhZVEl5Wm1Wa01USTBNemM0T0RWallnPT0iLA0KICAicXVlcnktbGFiZWwiIDogIlRlc3QgMyIsDQogICJxdWVyeS1leGVjdXRpb24tY29udGFjdC1pZCIgOiAiYmstYWRtaW5AdGVzdC5kZmt6LmRlIiwNCiAgInF1ZXJ5LWRlc2NyaXB0aW9uIiA6ICJUaGlzIGlzIHRoZSB0ZXN0IDMiLA0KICAicXVlcnktZXhwaXJhdGlvbi1kYXRlIiA6ICIyMDI0LTA4LTE0IiwNCiAgIm91dHB1dC1mb3JtYXQiIDogIkVYQ0VMIiwNCiAgInF1ZXJ5IiA6ICJleUpzWVc1bklqb2lZM0ZzSWl3aWJHbGlJanA3SW5KbGMyOTFjbU5sVkhsd1pTSTZJa3hwWW5KaGNua2lMQ0oxY213aU9pSjFjbTQ2ZFhWcFpEcGpOelJrWmpJd05DMDFZalppTFRSaFpXUXRZakl5T0MwM1pqVXpNekE0TnpZME5UZ2lMQ0p6ZEdGMGRYTWlPaUpoWTNScGRtVWlMQ0owZVhCbElqcDdJbU52WkdsdVp5STZXM3NpYzNsemRHVnRJam9pYUhSMGNEb3ZMM1JsY20xcGJtOXNiMmQ1TG1oc055NXZjbWN2UTI5a1pWTjVjM1JsYlM5c2FXSnlZWEo1TFhSNWNHVWlMQ0pqYjJSbElqb2liRzluYVdNdGJHbGljbUZ5ZVNKOVhYMHNJbU52Ym5SbGJuUWlPbHQ3SW1OdmJuUmxiblJVZVhCbElqb2lkR1Y0ZEM5amNXd2lMQ0prWVhSaElqb2lZa2RzYVdOdFJubGxVMEpUV2xoU2VXRlhWakphVVhBeFl6SnNkVnA1UWtkVFJXeFRTVWhhYkdOdVRuQmlNalJuU25wUmRVMUROSGRLZDNCd1ltMU9jMlJYVW14SlJWcEpVMVpLU1ZwWGVIZGFXRXA2U1VoYWJHTnVUbkJpTWpSblNucFJkVTFETkhkS2QyOUxXVEk1YTFwWVRqVmpNMUpzWWxOQ2MySXliSFZaZW05blNqSm9NR1JJUVRaTWVUbHpZakpzZFZsNU5YWmpiV051UTJkd2FtSXlOVEJhV0dnd1NVWkNhR1JIYkd4aWJsRkxRMmR3UlZNeFVreFlNVTVWVld0R1ZWZ3daRVpVYTFKR1ZXdzVWRlpHU2tKV1JXeEhVMVZXVTBObmNFVlRNVkpNV0RGT1ZWVnJSbFZZTVVKVFUxVXhRbFZzYkdaU1JXeENVakExVUZVd2JGUllNVTVWVld0R1ZWTlZXa3BTVmtsTFVrVjBWVk14T1ZSV1JrcENWa1k1UWxJd1ZtWlJNSGhDVlRGT1psVXhVbE5SVmxKS1VtdHNSbFZuYjB0U1JYUlZVekU1VkZaR1NrSldSamxGVWxWT1JsRldUa1pTUmpsVVZrWktRbFpGYkVkVFZWWlRRMmR3UlZNeFVreFlNVTVWVld0R1ZWZ3dVa3BSVldSUFZERk9TbFV4T1ZSV1JrcENWa1ZzUjFOVlZsTkRaM0JGVXpGU1RGZ3hUbFZWYTBaVldERk9VVkpWVGtwVVZWWlBXREZPVlZWclJsVlRWVnBLVWxaSlMwTnJVa3hXUlhSbVZURlNVMUZXVW1aVlJrcFFVVEJXUlZaV1NrWllNVTVWVld0R1ZWTlZXa3BTVmtsTFEydFNURlpGZEdaVk1WSlRVVlpTWmxSVlZrVlRWVTVDVmtWc1VGUnNPVlJXUmtwQ1ZrVnNSMU5WVmxORGExSk1Wa1YwWmxVeFVsTlJWbEptVWtWV1IxZ3diRTlZTUd4UFUxWlNTbEZWZUdaVlJUbFJWbFY0UWxaRmJGQlViRUpvWkVkc2JHSnVVWFZhTWxaMVdrZFdlVWxFTUdkS01qRm9Za2RWYmlKOVhYMHNJbTFsWVhOMWNtVWlPbnNpY21WemIzVnlZMlZVZVhCbElqb2lUV1ZoYzNWeVpTSXNJblZ5YkNJNkluVnlianAxZFdsa09qaG1NMlV6WVRZeExXRXdPVGN0TkRoa05DMWlOMkZqTFRobE5ESTNZbVU0WVdNMFpDSXNJbk4wWVhSMWN5STZJbUZqZEdsMlpTSXNJbk4xWW1wbFkzUkRiMlJsWVdKc1pVTnZibU5sY0hRaU9uc2lZMjlrYVc1bklqcGJleUp6ZVhOMFpXMGlPaUpvZEhSd09pOHZhR3czTG05eVp5OW1hR2x5TDNKbGMyOTFjbU5sTFhSNWNHVnpJaXdpWTI5a1pTSTZJbEJoZEdsbGJuUWlmVjE5TENKc2FXSnlZWEo1SWpvaWRYSnVPblYxYVdRNll6YzBaR1l5TURRdE5XSTJZaTAwWVdWa0xXSXlNamd0TjJZMU16TXdPRGMyTkRVNElpd2ljMk52Y21sdVp5STZleUpqYjJScGJtY2lPbHQ3SW5ONWMzUmxiU0k2SW1oMGRIQTZMeTkwWlhKdGFXNXZiRzluZVM1b2JEY3ViM0puTDBOdlpHVlRlWE4wWlcwdmJXVmhjM1Z5WlMxelkyOXlhVzVuSWl3aVkyOWtaU0k2SW1OdmFHOXlkQ0o5WFgwc0ltZHliM1Z3SWpwYmV5SmpiMlJsSWpwN0luUmxlSFFpT2lKd1lYUnBaVzUwY3lKOUxDSndiM0IxYkdGMGFXOXVJanBiZXlKamIyUmxJanA3SW1OdlpHbHVaeUk2VzNzaWMzbHpkR1Z0SWpvaWFIUjBjRG92TDNSbGNtMXBibTlzYjJkNUxtaHNOeTV2Y21jdlEyOWtaVk41YzNSbGJTOXRaV0Z6ZFhKbExYQnZjSFZzWVhScGIyNGlMQ0pqYjJSbElqb2lhVzVwZEdsaGJDMXdiM0IxYkdGMGFXOXVJbjFkZlN3aVkzSnBkR1Z5YVdFaU9uc2liR0Z1WjNWaFoyVWlPaUowWlhoMEwyTnhiQzFwWkdWdWRHbG1hV1Z5SWl3aVpYaHdjbVZ6YzJsdmJpSTZJa2x1U1c1cGRHbGhiRkJ2Y0hWc1lYUnBiMjRpZlgxZExDSnpkSEpoZEdsbWFXVnlJanBiZXlKamIyUmxJanA3SW5SbGVIUWlPaUpIWlc1a1pYSWlmU3dpWTNKcGRHVnlhV0VpT25zaWJHRnVaM1ZoWjJVaU9pSjBaWGgwTDJOeGJDSXNJbVY0Y0hKbGMzTnBiMjRpT2lKSFpXNWtaWElpZlgwc2V5SmpiMlJsSWpwN0luUmxlSFFpT2lJM05URTROaTAzSW4wc0ltTnlhWFJsY21saElqcDdJbXhoYm1kMVlXZGxJam9pZEdWNGRDOWpjV3dpTENKbGVIQnlaWE56YVc5dUlqb2lSR1ZqWldGelpXUWlmWDBzZXlKamIyUmxJanA3SW5SbGVIUWlPaUpCWjJVaWZTd2lZM0pwZEdWeWFXRWlPbnNpYkdGdVozVmhaMlVpT2lKMFpYaDBMMk54YkNJc0ltVjRjSEpsYzNOcGIyNGlPaUpCWjJWRGJHRnpjeUo5ZlYxOUxIc2lZMjlrWlNJNmV5SjBaWGgwSWpvaVpHbGhaMjV2YzJsekluMHNJbVY0ZEdWdWMybHZiaUk2VzNzaWRYSnNJam9pYUhSMGNEb3ZMMmhzTnk1dmNtY3ZabWhwY2k5MWN5OWpjV1p0WldGemRYSmxjeTlUZEhKMVkzUjFjbVZFWldacGJtbDBhVzl1TDJOeFptMHRjRzl3ZFd4aGRHbHZia0poYzJseklpd2lkbUZzZFdWRGIyUmxJam9pUTI5dVpHbDBhVzl1SW4xZExDSndiM0IxYkdGMGFXOXVJanBiZXlKamIyUmxJanA3SW1OdlpHbHVaeUk2VzNzaWMzbHpkR1Z0SWpvaWFIUjBjRG92TDNSbGNtMXBibTlzYjJkNUxtaHNOeTV2Y21jdlEyOWtaVk41YzNSbGJTOXRaV0Z6ZFhKbExYQnZjSFZzWVhScGIyNGlMQ0pqYjJSbElqb2lhVzVwZEdsaGJDMXdiM0IxYkdGMGFXOXVJbjFkZlN3aVkzSnBkR1Z5YVdFaU9uc2liR0Z1WjNWaFoyVWlPaUowWlhoMEwyTnhiQzFwWkdWdWRHbG1hV1Z5SWl3aVpYaHdjbVZ6YzJsdmJpSTZJa1JwWVdkdWIzTnBjeUo5ZlYwc0luTjBjbUYwYVdacFpYSWlPbHQ3SW1OdlpHVWlPbnNpZEdWNGRDSTZJbVJwWVdkdWIzTnBjeUo5TENKamNtbDBaWEpwWVNJNmV5SnNZVzVuZFdGblpTSTZJblJsZUhRdlkzRnNMV2xrWlc1MGFXWnBaWElpTENKbGVIQnlaWE56YVc5dUlqb2lSR2xoWjI1dmMybHpRMjlrWlNKOWZWMTlMSHNpWTI5a1pTSTZleUowWlhoMElqb2ljM0JsWTJsdFpXNGlmU3dpWlhoMFpXNXphVzl1SWpwYmV5SjFjbXdpT2lKb2RIUndPaTh2YUd3M0xtOXlaeTltYUdseUwzVnpMMk54Wm0xbFlYTjFjbVZ6TDFOMGNuVmpkSFZ5WlVSbFptbHVhWFJwYjI0dlkzRm1iUzF3YjNCMWJHRjBhVzl1UW1GemFYTWlMQ0oyWVd4MVpVTnZaR1VpT2lKVGNHVmphVzFsYmlKOVhTd2ljRzl3ZFd4aGRHbHZiaUk2VzNzaVkyOWtaU0k2ZXlKamIyUnBibWNpT2x0N0luTjVjM1JsYlNJNkltaDBkSEE2THk5MFpYSnRhVzV2Ykc5bmVTNW9iRGN1YjNKbkwwTnZaR1ZUZVhOMFpXMHZiV1ZoYzNWeVpTMXdiM0IxYkdGMGFXOXVJaXdpWTI5a1pTSTZJbWx1YVhScFlXd3RjRzl3ZFd4aGRHbHZiaUo5WFgwc0ltTnlhWFJsY21saElqcDdJbXhoYm1kMVlXZGxJam9pZEdWNGRDOWpjV3d0YVdSbGJuUnBabWxsY2lJc0ltVjRjSEpsYzNOcGIyNGlPaUpUY0dWamFXMWxiaUo5ZlYwc0luTjBjbUYwYVdacFpYSWlPbHQ3SW1OdlpHVWlPbnNpZEdWNGRDSTZJbk5oYlhCc1pWOXJhVzVrSW4wc0ltTnlhWFJsY21saElqcDdJbXhoYm1kMVlXZGxJam9pZEdWNGRDOWpjV3dpTENKbGVIQnlaWE56YVc5dUlqb2lVMkZ0Y0d4bFZIbHdaU0o5ZlYxOUxIc2lZMjlrWlNJNmV5SjBaWGgwSWpvaWNISnZZMlZrZFhKbGN5SjlMQ0psZUhSbGJuTnBiMjRpT2x0N0luVnliQ0k2SW1oMGRIQTZMeTlvYkRjdWIzSm5MMlpvYVhJdmRYTXZZM0ZtYldWaGMzVnlaWE12VTNSeWRXTjBkWEpsUkdWbWFXNXBkR2x2Ymk5amNXWnRMWEJ2Y0hWc1lYUnBiMjVDWVhOcGN5SXNJblpoYkhWbFEyOWtaU0k2SWxCeWIyTmxaSFZ5WlNKOVhTd2ljRzl3ZFd4aGRHbHZiaUk2VzNzaVkyOWtaU0k2ZXlKamIyUnBibWNpT2x0N0luTjVjM1JsYlNJNkltaDBkSEE2THk5MFpYSnRhVzV2Ykc5bmVTNW9iRGN1YjNKbkwwTnZaR1ZUZVhOMFpXMHZiV1ZoYzNWeVpTMXdiM0IxYkdGMGFXOXVJaXdpWTI5a1pTSTZJbWx1YVhScFlXd3RjRzl3ZFd4aGRHbHZiaUo5WFgwc0ltTnlhWFJsY21saElqcDdJbXhoYm1kMVlXZGxJam9pZEdWNGRDOWpjV3d0YVdSbGJuUnBabWxsY2lJc0ltVjRjSEpsYzNOcGIyNGlPaUpRY205alpXUjFjbVVpZlgxZExDSnpkSEpoZEdsbWFXVnlJanBiZXlKamIyUmxJanA3SW5SbGVIUWlPaUpRY205alpXUjFjbVZVZVhCbEluMHNJbU55YVhSbGNtbGhJanA3SW14aGJtZDFZV2RsSWpvaWRHVjRkQzlqY1d3aUxDSmxlSEJ5WlhOemFXOXVJam9pVUhKdlkyVmtkWEpsVkhsd1pTSjlmVjE5TEhzaVkyOWtaU0k2ZXlKMFpYaDBJam9pYldWa2FXTmhkR2x2YmxOMFlYUmxiV1Z1ZEhNaWZTd2laWGgwWlc1emFXOXVJanBiZXlKMWNtd2lPaUpvZEhSd09pOHZhR3czTG05eVp5OW1hR2x5TDNWekwyTnhabTFsWVhOMWNtVnpMMU4wY25WamRIVnlaVVJsWm1sdWFYUnBiMjR2WTNGbWJTMXdiM0IxYkdGMGFXOXVRbUZ6YVhNaUxDSjJZV3gxWlVOdlpHVWlPaUpOWldScFkyRjBhVzl1VTNSaGRHVnRaVzUwSW4xZExDSndiM0IxYkdGMGFXOXVJanBiZXlKamIyUmxJanA3SW1OdlpHbHVaeUk2VzNzaWMzbHpkR1Z0SWpvaWFIUjBjRG92TDNSbGNtMXBibTlzYjJkNUxtaHNOeTV2Y21jdlEyOWtaVk41YzNSbGJTOXRaV0Z6ZFhKbExYQnZjSFZzWVhScGIyNGlMQ0pqYjJSbElqb2lhVzVwZEdsaGJDMXdiM0IxYkdGMGFXOXVJbjFkZlN3aVkzSnBkR1Z5YVdFaU9uc2liR0Z1WjNWaFoyVWlPaUowWlhoMEwyTnhiQzFwWkdWdWRHbG1hV1Z5SWl3aVpYaHdjbVZ6YzJsdmJpSTZJazFsWkdsallYUnBiMjVUZEdGMFpXMWxiblFpZlgxZExDSnpkSEpoZEdsbWFXVnlJanBiZXlKamIyUmxJanA3SW5SbGVIUWlPaUpOWldScFkyRjBhVzl1Vkhsd1pTSjlMQ0pqY21sMFpYSnBZU0k2ZXlKc1lXNW5kV0ZuWlNJNkluUmxlSFF2WTNGc0lpd2laWGh3Y21WemMybHZiaUk2SWxCeWIyTmxaSFZ5WlZSNWNHVWlmWDFkZlYxOWZRPT0iLA0KICAicXVlcnktY29udGFjdC1pZCIgOiAicmVzZWFyY2hlckB0ZXN0LmRrZnouZGUiLA0KICAicXVlcnktZm9ybWF0IiA6ICJDUUxfREFUQSIsDQogICJ0ZW1wbGF0ZS1pZCIgOiAiY2NwIg0KfQ==","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"from":"app1.proxy1.broker","id":"22e1ea3a-07f3-4592-a888-82f2226a44a2","metadata":{"project":"exporter","task_type":"EXECUTE"},"to":["app1.proxy1.broker"],"ttl":"10s","status":null,"task":null}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks

```

Creating a sample [Exporter](https://github.com/samply/exporter) "status" task using curl:

```bash
curl -v -X POST -H "Content-Type: application/json" --data '{"body":"ew0KICAicXVlcnktZXhlY3V0aW9uLWlkIiA6ICIxOSINCn0=","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"from":"app1.proxy1.broker","id":"22e1ea3a-07f3-4592-a888-82f2226a44a2","metadata":{"project":"exporter","task_type":"STATUS"},"to":["app1.proxy1.broker"],"ttl":"10s","status":null,"task":null}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks

```

## License

This code is licensed under the Apache License 2.0. For details, please see [LICENSE](./LICENSE)
