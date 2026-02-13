//! LaTeX-like text rendering for mathematical expressions
//!
//! This module provides support for rendering LaTeX-like mathematical notation
//! in text labels, particularly useful for colorbar ticks and units.

use std::collections::HashMap;

/// Convert LaTeX-like syntax to Unicode mathematical symbols
pub fn latex_to_unicode(text: &str) -> String {
    let mut result = text.to_string();

    // Handle text formatting commands: \mathrm{...}, \mathbf{...}, etc.
    // These are stripped or simplified - we just keep the content
    result = handle_text_formatting(&result);

    // Greek letters (lowercase)
    let greek_lower: HashMap<&str, char> = [
        ("\\alpha", 'α'),
        ("\\beta", 'β'),
        ("\\gamma", 'γ'),
        ("\\delta", 'δ'),
        ("\\epsilon", 'ε'),
        ("\\zeta", 'ζ'),
        ("\\eta", 'η'),
        ("\\theta", 'θ'),
        ("\\iota", 'ι'),
        ("\\kappa", 'κ'),
        ("\\lambda", 'λ'),
        ("\\mu", 'μ'),
        ("\\nu", 'ν'),
        ("\\xi", 'ξ'),
        ("\\pi", 'π'),
        ("\\rho", 'ρ'),
        ("\\sigma", 'σ'),
        ("\\tau", 'τ'),
        ("\\upsilon", 'υ'),
        ("\\phi", 'φ'),
        ("\\chi", 'χ'),
        ("\\psi", 'ψ'),
        ("\\omega", 'ω'),
    ]
    .iter()
    .cloned()
    .collect();

    // Greek letters (uppercase)
    let greek_upper: HashMap<&str, char> = [
        ("\\Alpha", 'Α'),
        ("\\Beta", 'Β'),
        ("\\Gamma", 'Γ'),
        ("\\Delta", 'Δ'),
        ("\\Epsilon", 'Ε'),
        ("\\Zeta", 'Ζ'),
        ("\\Eta", 'Η'),
        ("\\Theta", 'Θ'),
        ("\\Iota", 'Ι'),
        ("\\Kappa", 'Κ'),
        ("\\Lambda", 'Λ'),
        ("\\Mu", 'Μ'),
        ("\\Nu", 'Ν'),
        ("\\Xi", 'Ξ'),
        ("\\Pi", 'Π'),
        ("\\Rho", 'Ρ'),
        ("\\Sigma", 'Σ'),
        ("\\Tau", 'Τ'),
        ("\\Upsilon", 'Υ'),
        ("\\Phi", 'Φ'),
        ("\\Chi", 'Χ'),
        ("\\Psi", 'Ψ'),
        ("\\Omega", 'Ω'),
    ]
    .iter()
    .cloned()
    .collect();

    // Mathematical operators and symbols
    let operators: HashMap<&str, &str> = [
        ("\\pm", "±"),
        ("\\mp", "∓"),
        ("\\times", "×"),
        ("\\div", "÷"),
        ("\\cdot", "⋅"),
        ("\\leq", "≤"),
        ("\\geq", "≥"),
        ("\\neq", "≠"),
        ("\\approx", "≈"),
        ("\\equiv", "≡"),
        ("\\propto", "∝"),
        ("\\infty", "∞"),
        ("\\partial", "∂"),
        ("\\nabla", "∇"),
        ("\\int", "∫"),
        ("\\sum", "∑"),
        ("\\prod", "∏"),
        ("\\sqrt", "√"),
        ("\\degree", "°"),
        ("\\angstrom", "Å"),
    ]
    .iter()
    .cloned()
    .collect();

    // Units
    let units: HashMap<&str, &str> = [
        ("\\mu m", "μm"),
        ("\\mu m", "μm"),
        ("\\degree", "°"),
        ("\\arcmin", "'"),
        ("\\arcsec", "\""),
        ("\\jansky", "Jy"),
        ("\\kelvin", "K"),
        ("\\hertz", "Hz"),
    ]
    .iter()
    .cloned()
    .collect();

    // Apply replacements in order of specificity (longest first)
    let mut all_replacements = Vec::new();

    // Add Greek letters
    for (latex, unicode) in greek_lower.iter().chain(greek_upper.iter()) {
        all_replacements.push((latex.to_string(), unicode.to_string()));
    }

    // Add operators
    for (latex, unicode) in operators.iter() {
        all_replacements.push((latex.to_string(), unicode.to_string()));
    }

    // Add units
    for (latex, unicode) in units.iter() {
        all_replacements.push((latex.to_string(), unicode.to_string()));
    }

    // Sort by length (longest first) to handle overlapping patterns
    all_replacements.sort_unstable_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (latex, unicode) in all_replacements {
        result = result.replace(&latex, &unicode);
    }

    // Handle superscripts: x^{2} -> x²
    result = handle_superscripts(&result);

    // Handle subscripts: x_{1} -> x₁ (basic support)
    result = handle_subscripts(&result);

    result
}

/// Handle LaTeX text formatting commands like \mathrm{...}, \mathbf{...}
/// Currently strips the formatting and keeps only the content
fn handle_text_formatting(text: &str) -> String {
    let mut result = text.to_string();

    // List of formatting commands to strip
    let commands = vec![
        "\\mathrm",
        "\\mathbf",
        "\\mathit",
        "\\mathtt",
        "\\mathsf",
        "\\mathnormal",
        "\\mathcal",
        "\\mathbb",
        "\\mathfrak",
        "\\mathscr",
        "\\text",
        "\\rm",
        "\\bf",
        "\\it",
        "\\tt",
    ];

    for cmd in commands {
        // Look for \mathrm{...} pattern and strip the command, keeping contents
        let pattern = format!("{}{{", cmd);
        while let Some(start) = result.find(&pattern) {
            if let Some(end) = result[start..].find('}') {
                let end_pos = start + end;
                let content = result[start + pattern.len()..end_pos].to_string();
                result.replace_range(start..=end_pos, &content);
            } else {
                break; // malformed, stop processing
            }
        }
    }

    result
}

/// Convert LaTeX superscript notation to Unicode superscripts
fn handle_superscripts(text: &str) -> String {
    let mut result = text.to_string();
    let superscript_map: HashMap<char, char> = [
        ('0', '⁰'),
        ('1', '¹'),
        ('2', '²'),
        ('3', '³'),
        ('4', '⁴'),
        ('5', '⁵'),
        ('6', '⁶'),
        ('7', '⁷'),
        ('8', '⁸'),
        ('9', '⁹'),
        ('+', '⁺'),
        ('-', '⁻'),
        ('=', '⁼'),
        ('(', '⁽'),
        (')', '⁾'),
        ('a', 'ᵃ'),
        ('b', 'ᵇ'),
        ('c', 'ᶜ'),
        ('d', 'ᵈ'),
        ('e', 'ᵉ'),
        ('f', 'ᶠ'),
        ('g', 'ᵍ'),
        ('h', 'ʰ'),
        ('i', 'ⁱ'),
        ('j', 'ʲ'),
        ('k', 'ᵏ'),
        ('l', 'ˡ'),
        ('m', 'ᵐ'),
        ('n', 'ⁿ'),
        ('o', 'ᵒ'),
        ('p', 'ᵖ'),
        ('q', '𐞥'),
        ('r', 'ʳ'),
        ('s', 'ˢ'),
        ('t', 'ᵗ'),
        ('u', 'ᵘ'),
        ('v', 'ᵛ'),
        ('w', 'ʷ'),
        ('x', 'ˣ'),
        ('y', 'ʸ'),
        ('z', 'ᶻ'),
    ]
    .iter()
    .cloned()
    .collect();

    // Handle ^{...} syntax
    while let Some(start) = result.find("^{") {
        if let Some(end) = result[start + 2..].find('}') {
            let end_pos = start + 2 + end;
            let superscript_content = &result[start + 2..end_pos];
            let mut unicode_sup = String::new();

            for ch in superscript_content.chars() {
                if let Some(sup_ch) = superscript_map.get(&ch) {
                    unicode_sup.push(*sup_ch);
                } else {
                    unicode_sup.push(ch); // fallback to original char
                }
            }

            result.replace_range(start..=end_pos, &unicode_sup);
        } else {
            break; // malformed, stop processing
        }
    }

    result
}

/// Convert LaTeX subscript notation to Unicode subscripts (limited support)
fn handle_subscripts(text: &str) -> String {
    let mut result = text.to_string();
    let subscript_map: HashMap<char, char> = [
        ('0', '₀'),
        ('1', '₁'),
        ('2', '₂'),
        ('3', '₃'),
        ('4', '₄'),
        ('5', '₅'),
        ('6', '₆'),
        ('7', '₇'),
        ('8', '₈'),
        ('9', '₉'),
        ('+', '₊'),
        ('-', '₋'),
        ('=', '₌'),
        ('(', '₍'),
        (')', '₎'),
        ('a', 'ₐ'),
        ('b', '♭'),
        ('c', '꜀'),
        ('d', 'ᷤ'),
        ('e', 'ₑ'),
        ('f', '꜀'),
        ('g', '₉'),
        ('h', 'ₕ'),
        ('i', 'ᵢ'),
        ('j', 'ⱼ'),
        ('k', 'ₖ'),
        ('l', 'ₗ'),
        ('m', 'ₘ'),
        ('n', 'ₙ'),
        ('o', 'ₒ'),
        ('p', 'ₚ'),
        ('q', 'ꝙ'),
        ('r', 'ᵣ'),
        ('s', 'ₛ'),
        ('t', 'ₜ'),
        ('u', 'ᵤ'),
        ('v', 'ᵥ'),
        ('w', 'ₓ'),
        ('x', 'ₓ'),
        ('y', 'ᵧ'),
        ('z', '₂'),
    ]
    .iter()
    .cloned()
    .collect();

    // Handle _{...} syntax
    while let Some(start) = result.find("_{") {
        if let Some(end) = result[start + 2..].find('}') {
            let end_pos = start + 2 + end;
            let subscript_content = &result[start + 2..end_pos];
            let mut unicode_sub = String::new();

            // Convert subscript content to lowercase for better Unicode support
            // (Unicode has more complete lowercase subscript coverage)
            for ch in subscript_content.to_lowercase().chars() {
                if let Some(sub_ch) = subscript_map.get(&ch) {
                    unicode_sub.push(*sub_ch);
                } else {
                    unicode_sub.push(ch); // fallback to original char
                }
            }

            result.replace_range(start..=end_pos, &unicode_sub);
        } else {
            break; // malformed, stop processing
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greek_letters() {
        assert_eq!(latex_to_unicode("\\alpha"), "α");
        assert_eq!(latex_to_unicode("\\beta"), "β");
        assert_eq!(latex_to_unicode("\\mu"), "μ");
        assert_eq!(latex_to_unicode("\\sigma"), "σ");
    }

    #[test]
    fn test_superscripts() {
        assert_eq!(latex_to_unicode("10^{2}"), "10²");
        assert_eq!(latex_to_unicode("x^{3}"), "x³");
        assert_eq!(latex_to_unicode("10^{-6}"), "10⁻⁶");
    }

    #[test]
    fn test_operators() {
        assert_eq!(latex_to_unicode("\\pm"), "±");
        assert_eq!(latex_to_unicode("\\times"), "×");
        assert_eq!(latex_to_unicode("\\leq"), "≤");
        assert_eq!(latex_to_unicode("\\infty"), "∞");
    }

    #[test]
    fn test_mixed_expressions() {
        assert_eq!(latex_to_unicode("\\mu m"), "μm");
        assert_eq!(latex_to_unicode("10^{-6} \\mu m"), "10⁻⁶ μm");
    }

    #[test]
    fn test_text_formatting() {
        // \mathrm should strip and keep content
        assert_eq!(latex_to_unicode("\\mathrm{K}"), "K");
        assert_eq!(latex_to_unicode("\\mathrm{CMB}"), "CMB");

        // Combined with greek letters
        assert_eq!(latex_to_unicode("\\mathrm{\\mu K}"), "μ K");

        // Subscripts with uppercase letters get converted to lowercase
        // (Unicode has better coverage for lowercase subscripts)
        let result = latex_to_unicode("K_{CMB}");
        assert!(result.contains("K")); // Base stays as K
        // CMB is converted to lowercase then subscripted
        assert!(!result.contains("_{")); // The _{...} syntax should be gone
    }
}
