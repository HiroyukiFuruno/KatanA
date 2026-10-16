use super::DocumentFontRequestsOps;

#[test]
fn empty_and_blank_cells_do_not_request_fonts() {
    let mut grid = super::painter_tests::grid_surface();
    grid.cells[0].text.clear();
    assert!(DocumentFontRequestsOps::grid_requests(&grid).is_empty());
    grid.cells[0].text = "visible".to_owned();
    grid.cells[0].appearance.font_family = " ".to_owned();
    assert!(DocumentFontRequestsOps::grid_requests(&grid).is_empty());
}

#[test]
fn family_case_is_deduplicated_but_actual_styles_are_distinct() {
    let mut grid = super::painter_tests::grid_surface();
    let mut regular = grid.cells[0].clone();
    regular.appearance.font_family = " Arial ".to_owned();
    let mut same = regular.clone();
    same.appearance.font_family = "arial".to_owned();
    let mut bold = regular.clone();
    bold.appearance.bold = true;
    let mut italic = regular.clone();
    italic.appearance.italic = true;
    grid.cells = vec![regular, same, bold, italic];
    let requests = DocumentFontRequestsOps::grid_requests(&grid);
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|request| request.family == "arial"));
    assert!(
        requests
            .iter()
            .any(|request| request.bold && !request.italic)
    );
    assert!(
        requests
            .iter()
            .any(|request| !request.bold && request.italic)
    );
}

#[test]
fn unicode_family_case_is_projected_to_one_request() {
    let mut grid = super::painter_tests::grid_surface();
    let mut upper = grid.cells[0].clone();
    upper.appearance.font_family = "École".to_owned();
    let mut lower = upper.clone();
    lower.appearance.font_family = "école".to_owned();
    grid.cells = vec![upper, lower];
    let requests = DocumentFontRequestsOps::grid_requests(&grid);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].family, "e\u{301}cole");
}

#[test]
fn canonically_equivalent_unicode_families_are_deduplicated() {
    let mut grid = super::painter_tests::grid_surface();
    let mut composed = grid.cells[0].clone();
    composed.appearance.font_family = "École".to_owned();
    let mut decomposed = composed.clone();
    decomposed.appearance.font_family = "E\u{301}cole".to_owned();
    grid.cells = vec![composed, decomposed];
    assert_eq!(DocumentFontRequestsOps::grid_requests(&grid).len(), 1);
}

#[test]
fn default_case_fold_deduplicates_sharp_s_and_ascii_ss() {
    let mut grid = super::painter_tests::grid_surface();
    let mut sharp_s = grid.cells[0].clone();
    sharp_s.appearance.font_family = "Straße".to_owned();
    let mut ascii_ss = sharp_s.clone();
    ascii_ss.appearance.font_family = "STRASSE".to_owned();
    grid.cells = vec![sharp_s, ascii_ss];
    assert_eq!(DocumentFontRequestsOps::grid_requests(&grid).len(), 1);
}

#[test]
fn styled_family_requests_include_regular_fallback_face() {
    let mut grid = super::painter_tests::grid_surface();
    grid.cells[0].appearance.font_family = "Arial".to_owned();
    grid.cells[0].appearance.bold = true;
    grid.cells[0].appearance.italic = true;
    let requests = DocumentFontRequestsOps::grid_requests(&grid);
    assert!(
        requests
            .iter()
            .any(|request| { request.family == "arial" && !request.bold && !request.italic })
    );
}

#[test]
fn bold_and_italic_requests_project_only_painter_fallback_styles() {
    let mut grid = super::painter_tests::grid_surface();
    let mut regular = grid.cells[0].clone();
    regular.appearance.font_family = "Arial".to_owned();
    let mut bold = regular.clone();
    bold.appearance.bold = true;
    let mut italic = regular.clone();
    italic.appearance.italic = true;
    let mut bold_italic = regular.clone();
    bold_italic.appearance.bold = true;
    bold_italic.appearance.italic = true;
    grid.cells = vec![bold, italic, bold_italic];
    let requests = DocumentFontRequestsOps::grid_requests(&grid);
    assert_eq!(requests.len(), 4);
    for (bold, italic) in [(false, false), (true, false), (false, true), (true, true)] {
        assert!(requests.iter().any(|request| {
            request.family == "arial" && request.bold == bold && request.italic == italic
        }));
    }
}

#[test]
fn regular_requests_do_not_project_unrequested_styles() {
    let mut grid = super::painter_tests::grid_surface();
    grid.cells[0].appearance.font_family = "Arial".to_owned();
    let requests = DocumentFontRequestsOps::grid_requests(&grid);
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].bold && !requests[0].italic);
}

#[test]
fn single_style_requests_do_not_project_the_other_style() {
    let mut grid = super::painter_tests::grid_surface();
    let mut bold = grid.cells[0].clone();
    bold.appearance.font_family = "Arial".to_owned();
    bold.appearance.bold = true;
    let mut italic = bold.clone();
    italic.appearance.bold = false;
    italic.appearance.italic = true;
    grid.cells = vec![bold];
    let bold_requests = DocumentFontRequestsOps::grid_requests(&grid);
    assert_eq!(bold_requests.len(), 2);
    assert!(bold_requests.iter().all(|request| !request.italic));
    grid.cells = vec![italic];
    let italic_requests = DocumentFontRequestsOps::grid_requests(&grid);
    assert_eq!(italic_requests.len(), 2);
    assert!(italic_requests.iter().all(|request| !request.bold));
}
