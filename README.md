# Local Spot


## Description
Local Spot is a Samply component ran on the sites, which checks Beam Broker for tasks for the application on the site, runs the tasks agains the local Blaze store, and sends the results back to Beam Broker. 

## Installation
- from Samply/Bridgehead: https://github.com/samply/bridgehead/
- standalone ...
	- also install: 
		- at least one Beam Proxy: https://github.com/samply/beam/
		- blaze store: https://github.com/samply/blaze

## Configuration
BEAM_BASE_URL = "http://localhost:8081"
BLAZE_BASE_URL = "http://localhost:8089/fhir"
PROXY_ID = "proxy1.broker"
API_KEY = "App1Secret"
APP_ID = "app1"


## Usage
Creating a sample task:
```
curl -v -X POST -H "Content-Type: application/json" --data '{"id":"70c0aa90-bfcf-4312-a6af-42cbd57dc0b8","from":"app1.proxy1.broker","to":["app1.proxy1.broker"],"ttl":10000,"failure_strategy":{"retry":{"backoff_millisecs":1000,"max_tries":5}},"metadata":"The broker can read and use this field e.g., to apply filters on behalf of an app","body":"eyJsYW5nIjoiY3FsIiwibGliIjp7InJlc291cmNlVHlwZSI6IkxpYnJhcnkiLCJ1cmwiOiJ1cm46dXVpZDo2N2M4YTExNC05OTRkLTQ3NGEtOWUyMC00ZTFjMWUzNGE0ZDAiLCJzdGF0dXMiOiJhY3RpdmUiLCJ0eXBlIjp7ImNvZGluZyI6W3sic3lzdGVtIjoiaHR0cDovL3Rlcm1pbm9sb2d5LmhsNy5vcmcvQ29kZVN5c3RlbS9saWJyYXJ5LXR5cGUiLCJjb2RlIjoibG9naWMtbGlicmFyeSJ9XX0sImNvbnRlbnQiOlt7ImNvbnRlbnRUeXBlIjoidGV4dC9jcWwiLCJkYXRhIjoiYkdsaWNtRnllU0JTWlhSeWFXVjJaUXAxYzJsdVp5QkdTRWxTSUhabGNuTnBiMjRnSnpRdU1DNHdKd3BwYm1Oc2RXUmxJRVpJU1ZKSVpXeHdaWEp6SUhabGNuTnBiMjRnSnpRdU1DNHdKd29LWTI5dWRHVjRkQ0JRWVhScFpXNTBDZ3BrWldacGJtVWdSMlZ1WkdWeU9ncFFZWFJwWlc1MExtZGxibVJsY2dvS1pHVm1hVzVsSUVSbFkyVmhjMlZrT2dwUVlYUnBaVzUwTG1SbFkyVmhjMlZrSUdseklHNXZkQ0J1ZFd4c0lBb0taR1ZtYVc1bElFRm5aVU5zWVhOek9nb29RV2RsU1c1WlpXRnljeWdwSUdScGRpQXhNQ2tnS2lBeE1Bb0taR1ZtYVc1bElFbHVTVzVwZEdsaGJGQnZjSFZzWVhScGIyNDZDblJ5ZFdVPSJ9XX0sIm1lYXN1cmUiOnsicmVzb3VyY2VUeXBlIjoiTWVhc3VyZSIsInVybCI6InVybjp1dWlkOmUyYTdkNWVkLTZkMGUtNDVmZC1hOGYzLTU2YWY1ZDUzNjc0OSIsInN0YXR1cyI6ImFjdGl2ZSIsInN1YmplY3RDb2RlYWJsZUNvbmNlcHQiOnsiY29kaW5nIjpbeyJzeXN0ZW0iOiJodHRwOi8vaGw3Lm9yZy9maGlyL3Jlc291cmNlLXR5cGVzIiwiY29kZSI6IlBhdGllbnQifV19LCJsaWJyYXJ5IjoidXJuOnV1aWQ6NjdjOGExMTQtOTk0ZC00NzRhLTllMjAtNGUxYzFlMzRhNGQwIiwic2NvcmluZyI6eyJjb2RpbmciOlt7InN5c3RlbSI6Imh0dHA6Ly90ZXJtaW5vbG9neS5obDcub3JnL0NvZGVTeXN0ZW0vbWVhc3VyZS1zY29yaW5nIiwiY29kZSI6ImNvaG9ydCJ9XX0sImdyb3VwIjpbeyJwb3B1bGF0aW9uIjpbeyJjb2RlIjp7ImNvZGluZyI6W3sic3lzdGVtIjoiaHR0cDovL3Rlcm1pbm9sb2d5LmhsNy5vcmcvQ29kZVN5c3RlbS9tZWFzdXJlLXBvcHVsYXRpb24iLCJjb2RlIjoiaW5pdGlhbC1wb3B1bGF0aW9uIn1dfSwiY3JpdGVyaWEiOnsibGFuZ3VhZ2UiOiJ0ZXh0L2NxbC1pZGVudGlmaWVyIiwiZXhwcmVzc2lvbiI6IkluSW5pdGlhbFBvcHVsYXRpb24ifX1dLCJzdHJhdGlmaWVyIjpbeyJjb2RlIjp7InRleHQiOiJHZW5kZXIifSwiY3JpdGVyaWEiOnsibGFuZ3VhZ2UiOiJ0ZXh0L2NxbCIsImV4cHJlc3Npb24iOiJHZW5kZXIifX0seyJjb2RlIjp7InRleHQiOiJEZWNlYXNlZCJ9LCJjcml0ZXJpYSI6eyJsYW5ndWFnZSI6InRleHQvY3FsIiwiZXhwcmVzc2lvbiI6IkRlY2Vhc2VkIn19LHsiY29kZSI6eyJ0ZXh0IjoiQWdlIn0sImNyaXRlcmlhIjp7Imxhbmd1YWdlIjoidGV4dC9jcWwiLCJleHByZXNzaW9uIjoiQWdlQ2xhc3MifX1dfV19fQo="}' -H "Authorization: ApiKey app1.proxy1.broker App1Secret" http://localhost:8081/v1/tasks
```

## License
Apache License 2.0

