use scriptui_core::{
    build_preview, default_theme, highlight_editor_line, parse_gfm, LineOverlay, SpellEngine,
};

#[test]
fn full_gfm_document_highlights_previews_and_spellchecks() {
    let markdown = concat!(
        "---\n",
        "title: ScripTUI\n",
        "---\n",
        "\n",
        "# Amber Editor\n",
        "\n",
        "Check this documnt with https://example.com/wrld and `inline_typo`.\n",
        "\n",
        "- [x] preview\n",
        "\n",
        "> [!NOTE]\n",
        "> block quote\n",
        "\n",
        "| Feature | Status |\n",
        "| :--- | :--- |\n",
        "| GFM | Ready |\n",
        "\n",
        "```rust\n",
        "let wrld = 1;\n",
        "```\n",
    );

    let doc = parse_gfm(markdown);
    let spell = SpellEngine::default_en_us().unwrap();
    let diagnostics = spell.check_document(markdown, &doc.spell_skip_byte_ranges);

    let flagged_words: Vec<&str> = diagnostics.iter().map(|d| d.word.as_str()).collect();
    assert!(
        flagged_words.contains(&"documnt"),
        "expected 'documnt' to be flagged, got {flagged_words:?}"
    );
    assert!(
        !flagged_words.contains(&"inline_typo"),
        "inline code should be skipped, got {flagged_words:?}"
    );
    assert!(
        !flagged_words.contains(&"wrld"),
        "URL path and fenced code block should be skipped, got {flagged_words:?}"
    );
    assert!(
        !flagged_words
            .iter()
            .any(|word| word.contains("example") || word.contains("https")),
        "autolink URL should be skipped, got {flagged_words:?}"
    );

    let suggestions = spell.suggest("documnt", 5);
    assert!(
        suggestions.iter().any(|s| s == "document"),
        "expected 'document' in suggestions for 'documnt', got {suggestions:?}"
    );

    let documnt_diag = diagnostics
        .iter()
        .find(|d| d.word == "documnt")
        .expect("missing diagnostic for documnt");
    let typo_line_idx = documnt_diag.line;
    let typo_line_text = markdown
        .lines()
        .nth(typo_line_idx)
        .expect("typo line should exist");
    assert_eq!(
        typo_line_text,
        "Check this documnt with https://example.com/wrld and `inline_typo`."
    );

    let theme = default_theme().unwrap();
    let spell_error_cols = diagnostics
        .iter()
        .filter(|d| d.line == typo_line_idx)
        .map(|d| d.col_start..d.col_end)
        .collect();
    let overlay = LineOverlay {
        spell_error_cols,
        ..Default::default()
    };
    let segments = highlight_editor_line(typo_line_idx, typo_line_text, &doc, &theme, &overlay);
    let spell_segment = segments
        .iter()
        .find(|seg| seg.text == "documnt")
        .expect("expected a styled segment for 'documnt'");
    assert_eq!(spell_segment.fg, theme.palette.spell_error);
    assert!(spell_segment.flags.underline);

    let preview = build_preview(markdown, &theme, 60);
    let plain = preview.plain_lines().join("\n");

    assert!(
        plain.contains('┌') && plain.contains('│') && plain.contains('└'),
        "expected box-drawing table borders in preview:\n{plain}"
    );
    assert!(
        plain.contains("[NOTE]"),
        "expected '[NOTE]' alert header in preview:\n{plain}"
    );
    assert!(
        plain.contains("[x]"),
        "expected '[x]' task marker in preview:\n{plain}"
    );

    // Verify monotonic bidirectional scroll-sync mapping.
    let line_count = markdown.lines().count();
    let mut prev_preview_line = 0usize;
    for source_line in 0..line_count {
        let mapped = preview.source_line_to_preview_line(source_line);
        assert!(
            mapped >= prev_preview_line,
            "source_line_to_preview_line not monotonic at source_line {source_line}: {mapped} < {prev_preview_line}"
        );
        prev_preview_line = mapped;
    }

    let mut prev_source_line = 0usize;
    for preview_line in 0..preview.lines.len() {
        let mapped = preview.preview_line_to_source_line(preview_line);
        assert!(
            mapped >= prev_source_line,
            "preview_line_to_source_line not monotonic at preview_line {preview_line}: {mapped} < {prev_source_line}"
        );
        prev_source_line = mapped;
    }
}
