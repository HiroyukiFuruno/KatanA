pub const SIMPLE_DRAWIO_XML: &str = r#"<mxfile><diagram name="test"><mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Box A" style="rounded=1;fillColor=#fff2cc;strokeColor=#d6b656;" vertex="1" parent="1">
    <mxGeometry x="80" y="80" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="Box B" vertex="1" parent="1">
    <mxGeometry x="280" y="80" width="120" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel></diagram></mxfile>"#;

pub const COMPRESSED_DRAWIO_XML: &str = r#"<mxfile><diagram name="compressed">hVFBDsIgEHwN57ZgfEDR9OTJFxDZFBIoBKilv5fKqtGk8bAJMzsz2V0I4zYPQXh1cRIMYWfCeHAu1ZfNHIwhtNWSsBOhtC3V7PS6UgV6EWBK/+W0yu/CzFAZ7qwPECPIwvcuV0FMq0FBcPMkYfN3hPWL0gmuXty27lJ2KJxK1mAb4yEkyLsDPimcbwBnIYW1SNBwaKtj/YaLlkmhnyKnQI8KQ4/IiVjx+A5+HaPBa1TwuXfz8xkP</diagram></mxfile>"#;

pub const IMAGE_DRAWIO_XML: &str = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="" style="shape=image;image=img/lib/ibm/miscellaneous/cognitive_services.svg;aspect=fixed;html=1;" vertex="1" parent="1">
    <mxGeometry x="40" y="40" width="80" height="80" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;

pub const OFFICIAL_STENCIL_DRAWIO_XML: &str = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="" style="shape=mxgraph.basic.oval_callout;fillColor=#6c8ebf;strokeColor=#6c8ebf;html=1;" vertex="1" parent="1">
    <mxGeometry x="40" y="40" width="120" height="120" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;

pub const BASIC_DRAWIO_FIXTURES: &[(&str, &str)] = &[
    (
        "01-empty-mxfile.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/01-empty-mxfile.drawio"),
    ),
    (
        "02-standalone-mxgraphmodel.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/02-standalone-mxgraphmodel.drawio"),
    ),
    (
        "03-basic-flow.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/03-basic-flow.drawio"),
    ),
    (
        "04-shape-style-matrix.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/04-shape-style-matrix.drawio"),
    ),
    (
        "05-edge-variants.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/05-edge-variants.drawio"),
    ),
    (
        "06-multi-page.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/06-multi-page.drawio"),
    ),
    (
        "07-html-labels-and-entities.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/07-html-labels-and-entities.drawio"),
    ),
    (
        "08-group-container.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/08-group-container.drawio"),
    ),
    (
        "09-layers-and-swimlane.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/09-layers-and-swimlane.drawio"),
    ),
    (
        "10-userobject-metadata.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/10-userobject-metadata.drawio"),
    ),
    (
        "11-japanese-labels.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/11-japanese-labels.drawio"),
    ),
    (
        "12-vars-placeholders.drawio",
        include_str!("../../../../assets/fixtures/drawio/basic/12-vars-placeholders.drawio"),
    ),
];
