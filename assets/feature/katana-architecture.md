```drawio
<mxfile host="app.diagrams.net" agent="Katana-AI" version="21.0.0">
  <diagram id="katana-arch" name="Katana Architecture">
    <mxGraphModel dx="1000" dy="800" grid="1" gridSize="10" guides="1" tooltips="1" connect="1" arrows="1" fold="1" page="1" pageScale="1" pageWidth="827" pageHeight="1169" math="0" shadow="0">
      <root>
        <mxCell id="0" />
        <mxCell id="1" parent="0" />

        <!-- App Core -->
        <mxCell id="core" value="Katana Core (Rust)" style="rounded=1;whiteSpace=wrap;html=1;fillColor=#dae8fc;strokeColor=#6c8ebf;fontStyle=1;fontSize=14;shadow=1;" vertex="1" parent="1">
          <mxGeometry x="310" y="220" width="200" height="80" as="geometry" />
        </mxCell>

        <!-- UI -->
        <mxCell id="ui" value="Katana UI (egui)" style="rounded=1;whiteSpace=wrap;html=1;fillColor=#d5e8d4;strokeColor=#82b366;fontStyle=1;fontSize=14;shadow=1;" vertex="1" parent="1">
          <mxGeometry x="310" y="80" width="200" height="60" as="geometry" />
        </mxCell>

        <!-- Components -->
        <mxCell id="linter" value="AST Linter&lt;br&gt;(pulldown-cmark)" style="shape=module;align=left;spacingLeft=20;align=center;verticalAlign=top;whiteSpace=wrap;html=1;fillColor=#ffe6cc;strokeColor=#d79b00;shadow=1;" vertex="1" parent="1">
          <mxGeometry x="120" y="380" width="140" height="80" as="geometry" />
        </mxCell>

        <mxCell id="preview" value="Preview Pane" style="shape=module;align=left;spacingLeft=20;align=center;verticalAlign=top;whiteSpace=wrap;html=1;fillColor=#fff2cc;strokeColor=#d6b656;shadow=1;" vertex="1" parent="1">
          <mxGeometry x="340" y="380" width="140" height="80" as="geometry" />
        </mxCell>

        <mxCell id="chrome" value="Headless Chrome&lt;br&gt;(Draw.io/Mermaid Renderer)" style="ellipse;whiteSpace=wrap;html=1;fillColor=#f8cecc;strokeColor=#b85450;shadow=1;" vertex="1" parent="1">
          <mxGeometry x="320" y="520" width="180" height="80" as="geometry" />
        </mxCell>

        <mxCell id="fs" value="File System&lt;br&gt;(Workspace)" style="shape=cylinder3;whiteSpace=wrap;html=1;boundedLbl=1;backgroundOutline=1;size=15;fillColor=#e1d5e7;strokeColor=#9673a6;shadow=1;" vertex="1" parent="1">
          <mxGeometry x="580" y="370" width="100" height="100" as="geometry" />
        </mxCell>

        <!-- Edges -->
        <mxCell id="edge1" parent="1" source="ui" target="core" edge="1">
          <mxGeometry relative="1" as="geometry">
            <mxPoint as="sourcePoint" />
            <mxPoint as="targetPoint" />
          </mxGeometry>
        </mxCell>
        <mxCell id="edge2" parent="1" source="core" target="linter" edge="1">
          <mxGeometry relative="1" as="geometry">
            <mxPoint as="sourcePoint" />
            <mxPoint as="targetPoint" />
          </mxGeometry>
        </mxCell>
        <mxCell id="edge3" parent="1" source="core" target="preview" edge="1">
          <mxGeometry relative="1" as="geometry">
            <mxPoint as="sourcePoint" />
            <mxPoint as="targetPoint" />
          </mxGeometry>
        </mxCell>
        <mxCell id="edge4" parent="1" source="core" target="fs" edge="1">
          <mxGeometry relative="1" as="geometry">
            <mxPoint as="sourcePoint" />
            <mxPoint as="targetPoint" />
          </mxGeometry>
        </mxCell>
        <mxCell id="edge5" parent="1" source="preview" target="chrome" edge="1">
          <mxGeometry relative="1" as="geometry">
            <mxPoint as="sourcePoint" />
            <mxPoint as="targetPoint" />
          </mxGeometry>
        </mxCell>

      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```
