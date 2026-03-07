# fetchium-crewai

CrewAI tool adapters for Fetchium.

## Install

```bash
pip install fetchium-crewai
```

REST mode:

```bash
pip install fetchium-crewai[rest]
```

## Usage

```python
from fetchium_crewai import FetchiumTool, FetchiumResearchTool

search_tool = FetchiumTool(
    rest_base_url="https://api.fetchium.com",
    rest_api_key="fetchium_...",
)

research_tool = FetchiumResearchTool(
    rest_base_url="https://api.fetchium.com",
    rest_api_key="fetchium_...",
)
```

CLI mode works if `fetchium` is installed on PATH.
