import importlib
import json
import sys
import types
import unittest


def install_crewai_stubs():
    crewai = types.ModuleType("crewai")
    tools = types.ModuleType("crewai.tools")
    pydantic = types.ModuleType("pydantic")

    class BaseTool:
        pass

    class BaseModel:
        pass

    def Field(default=None, description=""):
        return default

    tools.BaseTool = BaseTool
    pydantic.BaseModel = BaseModel
    pydantic.Field = Field

    sys.modules["crewai"] = crewai
    sys.modules["crewai.tools"] = tools
    sys.modules["pydantic"] = pydantic


class FetchiumCrewAITests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        install_crewai_stubs()
        sys.path.insert(0, "adapters/crewai")
        cls.module = importlib.import_module("fetchium_crewai.tool")

    def test_format_rest_search_response(self):
        raw = json.dumps(
            {
                "results": [
                    {
                        "title": "Rust Book",
                        "snippet": "Learn Rust",
                        "url": "https://doc.rust-lang.org/book/",
                    }
                ]
            }
        )
        text = self.module.FetchiumTool._format_rest_search_response(raw)
        self.assertIn("Rust Book", text)
        self.assertIn("https://doc.rust-lang.org/book/", text)

    def test_format_rest_research_response(self):
        raw = json.dumps(
            {
                "report": "Rust is memory safe.",
                "reference_section": "[1] https://www.rust-lang.org/",
            }
        )
        text = self.module.FetchiumResearchTool._format_rest_research_response(raw)
        self.assertIn("Rust is memory safe.", text)
        self.assertIn("References:", text)


if __name__ == "__main__":
    unittest.main()
