# Data Model

This document contains an embedded Draw.io diagram representing the core data model.

## Entity Relationships

```drawio
<mxfile><diagram name="data-model"><mxGraphModel><root><mxCell id="0"/><mxCell id="1" parent="0"/><mxCell id="2" value="Workspace" style="rounded=1;" vertex="1" parent="1"><mxGeometry x="80" y="80" width="120" height="60" as="geometry"/></mxCell><mxCell id="3" value="Document" style="rounded=1;" vertex="1" parent="1"><mxGeometry x="280" y="80" width="120" height="60" as="geometry"/></mxCell><mxCell id="4" value="contains" edge="1" source="2" target="3" parent="1"><mxGeometry relative="1" as="geometry"/></mxCell></root></mxGraphModel></diagram></mxfile>
```

## Notes

- A `Workspace` is a local directory opened as the project root.
- A `Document` is an in-memory Markdown buffer loaded from a file in the workspace.
- Documents are saved explicitly; no implicit background writes occur.
