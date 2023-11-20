# Focus

Focus is a Samply component ran on the sites, which distributes tasks from Beam.Proxy to the applications on the site and re-transmits the results through Samply.Beam. Currenly, only Samply.Blaze is supported as a target application, but Focus is easily extensible.

It is possible to specify the queries whose results are to be cached to speed up retrieval. The cached results expire after 24 hours. 

## Installation

### Samply/Bridgehead Integration

Focus is already included in the [Samply.Bridgehead deployment](https://github.com/samply/bridgehead/), a turnkey solution for deploying, maintaining, and monitoring applications in a medical IT environment.

### Standalone Installation

To run a standalone Focus, you need at least one running [Samply.Beam.Proxy](https://github.com/samply/beam/) and one running [Samply.Blaze FHIR store](https://github.com/samply/blaze).
You can compile and run this application via Cargo, however, we encourage the usage of the pre-compiled [docker images](https://hub.docker.com/r/samply/focus):

```bash
docker run --rm -e BEAM_PROXY_URL=http://localhost:8081 -e ENDPOINT_URL=http://localhost:8089/fhir -e PROXY_ID=proxy1.broker -e API_KEY=App1Secret -e BEAM_APP_ID_LONG=app1.broker.example.com samply/focus:latest
```

## Configuration

The following environment variables are mandatory for the usage of Focus. If compiling and running Focus yourself, they are provided as command line options as well. See `focus  --help` for details.

```bash
BEAM_PROXY_URL = "http://localhost:8081" 
ENDPOINT_URL = "http://localhost:8089/fhir"
PROXY_ID = "proxy1.broker"
API_KEY = "App1Secret"
BEAM_APP_ID_LONG = "app1.broker.example.com"
```

### Optional variables

```bash
RETRY_COUNT = "32" # The maximum number of retries for beam and blaze healthchecks, default value: 32
ENDPOINT_TYPE = "blaze" # Type of the endpoint, allowed values: "blaze", "omop", default value: "blaze"
OBFUSCATE = "yes" # Should the results be obfuscated - the "master switch", allowed values: "yes", "no", default value: "yes"
OBFUSCATE_BELOW_10_MODE = "1" # The mode of obfuscating values below 10: 0 - return zero, 1 - return ten, 2 - obfuscate using Laplace distribution and rounding, has no effect if OBFUSCATE = "no", default value: 1
DELTA_PATIENT = "1." # Sensitivity parameter for obfuscating the counts in the Patient stratifier, has no effect if OBFUSCATE = "no", default value: 1
DELTA_SPECIMEN = "20." # Sensitivity parameter for obfuscating the counts in the Specimen stratifier, has no effect if OBFUSCATE = "no", default value: 20
DELTA_DIAGNOSIS = "3." # Sensitivity parameter for obfuscating the counts in the Diagnosis stratifier, has no effect if OBFUSCATE = "no", default value: 3
DELTA_PROCEDURES = "1.7" # Sensitivity parameter for obfuscating the counts in the Procedures stratifier, has no effect if OBFUSCATE = "no", default value: 1.7
DELTA_MEDICATION_STATEMENTS = "2.1" # Sensitivity parameter for obfuscating the counts in the Medication Statements stratifier, has no effect if OBFUSCATE = "no", default value: 2.1
EPSILON = "0.1" # Privacy budget parameter for obfuscating the counts in the stratifiers, has no effect if OBFUSCATE = "no", default value: 0.1
ROUNDING_STEP = "10" # The granularity of the rounding of the obfuscated values, has no effect if OBFUSCATE = "no", default value: 10
PROJECTS_NO_OBFUSCATION = "exliquid;dktk_supervisors" # Projects for which the results are not to be obfuscated, separated by ;, default value: "exliquid; dktk_supervisors"
QUERIES_TO_CACHE_FILE_PATH = "resources/bbmri" # The path to the file containing BASE64 encoded queries whose results are to be cached, if not set, no results are cached
PROVIDER = "name" #OMOP provider name
PROVIDER_ICON = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEUAAACnej3aAAAAAXRSTlMAQObYZgAAAApJREFUCNdjYAAAAAIAAeIhvDMAAAAASUVORK5CYII=" #Base64 encoded OMOP provider icon
AUTH_HEADER = "ApiKey XXXX" #Authorization header
```

Obfuscating zero counts is by default switched off. To enable obfuscating zero counts, set the env. variable `OBFUSCATE_ZERO`. 

Optionally, you can provide the `TLS_CA_CERTIFICATES_DIR` environment variable to add additional trusted certificates, e.g., if you have a TLS-terminating proxy server in place. The application respects the `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`, and their respective lowercase equivalents.

## Usage

Creating a sample task using CURL:

```bash
curl -v -X POST -H "Content-Type: application/json" --data '{"id":"7fffefff-ffef-fcff-feef-fefbffffeeff","from":"app1.proxy1.broker","to":["app1.proxy1.broker"],"ttl":"10s","failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"metadata":{"project":"exliquid"},"body":"ewoJImxhbmciOiAiY3FsIiwKCSJsaWIiOiB7CgkJImNvbnRlbnQiOiBbCgkJCXsKCQkJCSJjb250ZW50VHlwZSI6ICJ0ZXh0L2NxbCIsCgkJCQkiZGF0YSI6ICJiR2xpY21GeWVTQlNaWFJ5YVdWMlpRcDFjMmx1WnlCR1NFbFNJSFpsY25OcGIyNGdKelF1TUM0d0p3cHBibU5zZFdSbElFWklTVkpJWld4d1pYSnpJSFpsY25OcGIyNGdKelF1TUM0d0p3b0tZMjlrWlhONWMzUmxiU0JzYjJsdVl6b2dKMmgwZEhBNkx5OXNiMmx1WXk1dmNtY25DbU52WkdWemVYTjBaVzBnYVdOa01UQTZJQ2RvZEhSd09pOHZhR3czTG05eVp5OW1hR2x5TDNOcFpDOXBZMlF0TVRBbkNtTnZaR1Z6ZVhOMFpXMGdVMkZ0Y0d4bFRXRjBaWEpwWVd4VWVYQmxPaUFuYUhSMGNITTZMeTltYUdseUxtSmliWEpwTG1SbEwwTnZaR1ZUZVhOMFpXMHZVMkZ0Y0d4bFRXRjBaWEpwWVd4VWVYQmxKd29LQ21OdmJuUmxlSFFnVUdGMGFXVnVkQW9LUWtKTlVrbGZVMVJTUVZSZlIwVk9SRVZTWDFOVVVrRlVTVVpKUlZJS0NrSkNUVkpKWDFOVVVrRlVYMFJGUmw5VFVFVkRTVTFGVGdwcFppQkpia2x1YVhScFlXeFFiM0IxYkdGMGFXOXVJSFJvWlc0Z1cxTndaV05wYldWdVhTQmxiSE5sSUh0OUlHRnpJRXhwYzNROFUzQmxZMmx0Wlc0K0NncENRazFTU1Y5VFZGSkJWRjlUUVUxUVRFVmZWRmxRUlY5VFZGSkJWRWxHU1VWU0NncENRazFTU1Y5VFZGSkJWRjlEVlZOVVQwUkpRVTVmVTFSU1FWUkpSa2xGVWdvS1FrSk5Va2xmVTFSU1FWUmZSRWxCUjA1UFUwbFRYMU5VVWtGVVNVWkpSVklLQ2tKQ1RWSkpYMU5VVWtGVVgwRkhSVjlUVkZKQlZFbEdTVVZTQ2dwQ1FrMVNTVjlUVkZKQlZGOUVSVVpmU1U1ZlNVNUpWRWxCVEY5UVQxQlZURUZVU1U5T0NuUnlkV1U9IgoJCQl9CgkJXSwKCQkicmVzb3VyY2VUeXBlIjogIkxpYnJhcnkiLAoJCSJzdGF0dXMiOiAiYWN0aXZlIiwKCQkidHlwZSI6IHsKCQkJImNvZGluZyI6IFsKCQkJCXsKCQkJCQkiY29kZSI6ICJsb2dpYy1saWJyYXJ5IiwKCQkJCQkic3lzdGVtIjogImh0dHA6Ly90ZXJtaW5vbG9neS5obDcub3JnL0NvZGVTeXN0ZW0vbGlicmFyeS10eXBlIgoJCQkJfQoJCQldCgkJfSwKCQkidXJsIjogInVybjp1dWlkOjdmZjUzMmFkLTY5ZTQtNDhlZC1hMmQzLTllZmFmYjYwOWY2MiIKCX0sCgkibWVhc3VyZSI6IHsKCQkiZ3JvdXAiOiBbCgkJCXsKCQkJCSJjb2RlIjogewoJCQkJCSJ0ZXh0IjogInBhdGllbnRzIgoJCQkJfSwKCQkJCSJwb3B1bGF0aW9uIjogWwoJCQkJCXsKCQkJCQkJImNvZGUiOiB7CgkJCQkJCQkiY29kaW5nIjogWwoJCQkJCQkJCXsKCQkJCQkJCQkJImNvZGUiOiAiaW5pdGlhbC1wb3B1bGF0aW9uIiwKCQkJCQkJCQkJInN5c3RlbSI6ICJodHRwOi8vdGVybWlub2xvZ3kuaGw3Lm9yZy9Db2RlU3lzdGVtL21lYXN1cmUtcG9wdWxhdGlvbiIKCQkJCQkJCQl9CgkJCQkJCQldCgkJCQkJCX0sCgkJCQkJCSJjcml0ZXJpYSI6IHsKCQkJCQkJCSJleHByZXNzaW9uIjogIkluSW5pdGlhbFBvcHVsYXRpb24iLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsLWlkZW50aWZpZXIiCgkJCQkJCX0KCQkJCQl9CgkJCQldLAoJCQkJInN0cmF0aWZpZXIiOiBbCgkJCQkJewoJCQkJCQkiY29kZSI6IHsKCQkJCQkJCSJ0ZXh0IjogIkdlbmRlciIKCQkJCQkJfSwKCQkJCQkJImNyaXRlcmlhIjogewoJCQkJCQkJImV4cHJlc3Npb24iOiAiR2VuZGVyIiwKCQkJCQkJCSJsYW5ndWFnZSI6ICJ0ZXh0L2NxbCIKCQkJCQkJfQoJCQkJCX0sCgkJCQkJewoJCQkJCQkiY29kZSI6IHsKCQkJCQkJCSJ0ZXh0IjogIkFnZSIKCQkJCQkJfSwKCQkJCQkJImNyaXRlcmlhIjogewoJCQkJCQkJImV4cHJlc3Npb24iOiAiQWdlQ2xhc3MiLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsIgoJCQkJCQl9CgkJCQkJfSwKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJInRleHQiOiAiQ3VzdG9kaWFuIgoJCQkJCQl9LAoJCQkJCQkiY3JpdGVyaWEiOiB7CgkJCQkJCQkiZXhwcmVzc2lvbiI6ICJDdXN0b2RpYW4iLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsIgoJCQkJCQl9CgkJCQkJfQoJCQkJXQoJCQl9LAoJCQl7CgkJCQkiY29kZSI6IHsKCQkJCQkidGV4dCI6ICJkaWFnbm9zaXMiCgkJCQl9LAoJCQkJImV4dGVuc2lvbiI6IFsKCQkJCQl7CgkJCQkJCSJ1cmwiOiAiaHR0cDovL2hsNy5vcmcvZmhpci91cy9jcWZtZWFzdXJlcy9TdHJ1Y3R1cmVEZWZpbml0aW9uL2NxZm0tcG9wdWxhdGlvbkJhc2lzIiwKCQkJCQkJInZhbHVlQ29kZSI6ICJDb25kaXRpb24iCgkJCQkJfQoJCQkJXSwKCQkJCSJwb3B1bGF0aW9uIjogWwoJCQkJCXsKCQkJCQkJImNvZGUiOiB7CgkJCQkJCQkiY29kaW5nIjogWwoJCQkJCQkJCXsKCQkJCQkJCQkJImNvZGUiOiAiaW5pdGlhbC1wb3B1bGF0aW9uIiwKCQkJCQkJCQkJInN5c3RlbSI6ICJodHRwOi8vdGVybWlub2xvZ3kuaGw3Lm9yZy9Db2RlU3lzdGVtL21lYXN1cmUtcG9wdWxhdGlvbiIKCQkJCQkJCQl9CgkJCQkJCQldCgkJCQkJCX0sCgkJCQkJCSJjcml0ZXJpYSI6IHsKCQkJCQkJCSJleHByZXNzaW9uIjogIkRpYWdub3NpcyIsCgkJCQkJCQkibGFuZ3VhZ2UiOiAidGV4dC9jcWwtaWRlbnRpZmllciIKCQkJCQkJfQoJCQkJCX0KCQkJCV0sCgkJCQkic3RyYXRpZmllciI6IFsKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJInRleHQiOiAiZGlhZ25vc2lzIgoJCQkJCQl9LAoJCQkJCQkiY3JpdGVyaWEiOiB7CgkJCQkJCQkiZXhwcmVzc2lvbiI6ICJEaWFnbm9zaXNDb2RlIiwKCQkJCQkJCSJsYW5ndWFnZSI6ICJ0ZXh0L2NxbC1pZGVudGlmaWVyIgoJCQkJCQl9CgkJCQkJfQoJCQkJXQoJCQl9LAoJCQl7CgkJCQkiY29kZSI6IHsKCQkJCQkidGV4dCI6ICJzcGVjaW1lbiIKCQkJCX0sCgkJCQkiZXh0ZW5zaW9uIjogWwoJCQkJCXsKCQkJCQkJInVybCI6ICJodHRwOi8vaGw3Lm9yZy9maGlyL3VzL2NxZm1lYXN1cmVzL1N0cnVjdHVyZURlZmluaXRpb24vY3FmbS1wb3B1bGF0aW9uQmFzaXMiLAoJCQkJCQkidmFsdWVDb2RlIjogIlNwZWNpbWVuIgoJCQkJCX0KCQkJCV0sCgkJCQkicG9wdWxhdGlvbiI6IFsKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJImNvZGluZyI6IFsKCQkJCQkJCQl7CgkJCQkJCQkJCSJjb2RlIjogImluaXRpYWwtcG9wdWxhdGlvbiIsCgkJCQkJCQkJCSJzeXN0ZW0iOiAiaHR0cDovL3Rlcm1pbm9sb2d5LmhsNy5vcmcvQ29kZVN5c3RlbS9tZWFzdXJlLXBvcHVsYXRpb24iCgkJCQkJCQkJfQoJCQkJCQkJXQoJCQkJCQl9LAoJCQkJCQkiY3JpdGVyaWEiOiB7CgkJCQkJCQkiZXhwcmVzc2lvbiI6ICJTcGVjaW1lbiIsCgkJCQkJCQkibGFuZ3VhZ2UiOiAidGV4dC9jcWwtaWRlbnRpZmllciIKCQkJCQkJfQoJCQkJCX0KCQkJCV0sCgkJCQkic3RyYXRpZmllciI6IFsKCQkJCQl7CgkJCQkJCSJjb2RlIjogewoJCQkJCQkJInRleHQiOiAic2FtcGxlX2tpbmQiCgkJCQkJCX0sCgkJCQkJCSJjcml0ZXJpYSI6IHsKCQkJCQkJCSJleHByZXNzaW9uIjogIlNhbXBsZVR5cGUiLAoJCQkJCQkJImxhbmd1YWdlIjogInRleHQvY3FsIgoJCQkJCQl9CgkJCQkJfQoJCQkJXQoJCQl9CgkJXSwKCQkibGlicmFyeSI6ICJ1cm46dXVpZDo3ZmY1MzJhZC02OWU0LTQ4ZWQtYTJkMy05ZWZhZmI2MDlmNjIiLAoJCSJyZXNvdXJjZVR5cGUiOiAiTWVhc3VyZSIsCgkJInNjb3JpbmciOiB7CgkJCSJjb2RpbmciOiBbCgkJCQl7CgkJCQkJImNvZGUiOiAiY29ob3J0IiwKCQkJCQkic3lzdGVtIjogImh0dHA6Ly90ZXJtaW5vbG9neS5obDcub3JnL0NvZGVTeXN0ZW0vbWVhc3VyZS1zY29yaW5nIgoJCQkJfQoJCQldCgkJfSwKCQkic3RhdHVzIjogImFjdGl2ZSIsCgkJInN1YmplY3RDb2RlYWJsZUNvbmNlcHQiOiB7CgkJCSJjb2RpbmciOiBbCgkJCQl7CgkJCQkJImNvZGUiOiAiUGF0aWVudCIsCgkJCQkJInN5c3RlbSI6ICJodHRwOi8vaGw3Lm9yZy9maGlyL3Jlc291cmNlLXR5cGVzIgoJCQkJfQoJCQldCgkJfSwKCQkidXJsIjogInVybjp1dWlkOjVlZThkZTczLTM0N2UtNDdjYS1hMDE0LWYyZTcxNzY3YWRmYyIKCX0KfQ=="}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks
```

## License

This code is licensed under the Apache License 2.0. For details, please see [LICENSE](./LICENSE)
