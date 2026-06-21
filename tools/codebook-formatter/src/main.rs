use std::io::{self, BufRead, Write, BufWriter};
use unicode_general_category::{get_general_category, GeneralCategory};

// Escape a character for use in LaTeX body text with \obeylines\obeyspaces active.
// Only \, {, } retain special meaning in that mode; $, &, etc. are still special
// in normal LaTeX catcodes, so all 10 classic special chars need escaping.
// Control characters (Cc category) are replaced with a visible placeholder.
fn escape_char(ch: char) -> String {
    if ch.is_control() {
        return "\u{00B7}".to_string(); // MIDDLE DOT as placeholder
    }
    match ch {
        '\\' => "\\textbackslash{}".to_string(),
        '{' => "\\{".to_string(),
        '}' => "\\}".to_string(),
        '$' => "\\$".to_string(),
        '&' => "\\&".to_string(),
        '%' => "\\%".to_string(),
        '#' => "\\#".to_string(),
        '_' => "\\_".to_string(),
        '~' => "\\textasciitilde{}".to_string(),
        '^' => "\\textasciicircum{}".to_string(),
        c => c.to_string(),
    }
}

// Preamble for a LuaLaTeX document.
// Primary font: Noto Sans Mono.
// Fallback chain: Noto first (best quality, ordered by unique coverage),
// then FreeFont, DejaVu, quality CJK fonts, other purpose-specific fonts,
// and Unifont last (maximum utilitarian coverage).
// COVER_TITLE is replaced at runtime with the appropriate title string.
// Compile with: lualatex unicode-codebook.tex
const PREAMBLE_TEMPLATE: &str = r#"\documentclass[twoside]{article}
\usepackage[letterpaper,portrait,twoside,top=8mm,bottom=12mm,inner=20mm,outer=10mm]{geometry}
\usepackage{multicol}
\usepackage{fontspec}
\usepackage{fancyhdr}
\setlength{\footskip}{7mm}
\fancyhf{}
\fancyfoot[LE,RO]{\fontsize{7pt}{9pt}\selectfont\rmfamily\thepage}
\renewcommand{\headrulewidth}{0pt}
\renewcommand{\footrulewidth}{0pt}
\directlua{
  luaotfload.add_fallback("monofallback", {
    "Noto Sans CJK JP:mode=harf;",
    "Noto Serif CJK JP:mode=harf;",
    "HanaMinA:mode=harf;",
    "HanaMinB:mode=harf;",
    "BabelStone Han:mode=harf;",
    "Noto Serif Tangut:mode=harf;",
    "Noto Sans Symbols2:mode=harf;",
    "Noto Sans Math:mode=harf;",
    "Noto Color Emoji:mode=harf;",
    "Noto Sans Yi:mode=harf;",
    "Noto Sans Cuneiform:mode=harf;",
    "Noto Sans Arabic:mode=harf;",
    "Noto Naskh Arabic:mode=harf;",
    "Noto Sans Egyptian Hieroglyphs:mode=harf;",
    "Noto Sans Symbols:mode=harf;",
    "Noto Sans Canadian Aboriginal:mode=harf;",
    "Noto Sans SignWriting:mode=harf;",
    "Noto Sans Bamum:mode=harf;",
    "Noto Sans Anatolian Hieroglyphs:mode=harf;",
    "Noto Music:mode=harf;",
    "Noto Sans Ethiopic:mode=harf;",
    "Noto Serif Ethiopic:mode=harf;",
    "Noto Traditional Nushu:mode=harf;",
    "Noto Sans Nushu:mode=harf;",
    "Noto Sans Linear A:mode=harf;",
    "Noto Sans Vai:mode=harf;",
    "Noto Kufi Arabic:mode=harf;",
    "Noto Sans Linear B:mode=harf;",
    "Noto Nastaliq Urdu:mode=harf;",
    "Noto Sans Myanmar:mode=harf;",
    "Noto Serif Myanmar:mode=harf;",
    "Noto Sans Mende Kikakui:mode=harf;",
    "Noto Serif Tibetan:mode=harf;",
    "Noto Sans Devanagari:mode=harf;",
    "Noto Serif Devanagari:mode=harf;",
    "Noto Sans Mongolian:mode=harf;",
    "Noto Sans Cherokee:mode=harf;",
    "Noto Sans Georgian:mode=harf;",
    "Noto Serif Georgian:mode=harf;",
    "Noto Sans Miao:mode=harf;",
    "Noto Sans Duployan:mode=harf;",
    "Noto Sans Khmer:mode=harf;",
    "Noto Serif Khmer:mode=harf;",
    "Noto Sans Hebrew:mode=harf;",
    "Noto Serif Hebrew:mode=harf;",
    "Noto Sans Glagolitic:mode=harf;",
    "Noto Sans Tai Tham:mode=harf;",
    "Noto Sans Pahawh Hmong:mode=harf;",
    "Noto Sans Malayalam:mode=harf;",
    "Noto Serif Malayalam:mode=harf;",
    "Noto Sans Balinese:mode=harf;",
    "Noto Serif Balinese:mode=harf;",
    "Noto Sans Meroitic:mode=harf;",
    "Noto Sans Bengali:mode=harf;",
    "Noto Serif Bengali:mode=harf;",
    "Noto Sans Sinhala:mode=harf;",
    "Noto Serif Sinhala:mode=harf;",
    "Noto Sans Grantha:mode=harf;",
    "Noto Serif Grantha:mode=harf;",
    "Noto Sans Syriac:mode=harf;",
    "Noto Sans Brahmi:mode=harf;",
    "Noto Sans Old Hungarian:mode=harf;",
    "Noto Sans Gujarati:mode=harf;",
    "Noto Serif Gujarati:mode=harf;",
    "Noto Sans Kannada:mode=harf;",
    "Noto Serif Kannada:mode=harf;",
    "Noto Sans Telugu:mode=harf;",
    "Noto Serif Telugu:mode=harf;",
    "Noto Sans Sharada:mode=harf;",
    "Noto Sans Tirhuta:mode=harf;",
    "Noto Looped Thai:mode=harf;",
    "Noto Sans Armenian:mode=harf;",
    "Noto Serif Armenian:mode=harf;",
    "Noto Sans Newa:mode=harf;",
    "Noto Sans Bhaiksuki:mode=harf;",
    "Noto Sans Gurmukhi:mode=harf;",
    "Noto Serif Gurmukhi:mode=harf;",
    "Noto Sans Oriya:mode=harf;",
    "Noto Sans Siddham:mode=harf;",
    "Noto Sans Javanese:mode=harf;",
    "Noto Sans Medefaidrin:mode=harf;",
    "Noto Sans Adlam:mode=harf;",
    "Noto Sans Chakma:mode=harf;",
    "Noto Sans Runic:mode=harf;",
    "Noto Sans Modi:mode=harf;",
    "Noto Sans Indic Siyaq Numbers:mode=harf;",
    "Noto Sans Kaithi:mode=harf;",
    "Noto Rashi Hebrew:mode=harf;",
    "Noto Sans Thai:mode=harf;",
    "Noto Serif Thai:mode=harf;",
    "Noto Serif Khojki:mode=harf;",
    "Noto Looped Lao:mode=harf;",
    "Noto Sans Warang Citi:mode=harf;",
    "Noto Sans Soyombo:mode=harf;",
    "Noto Sans New Tai Lue:mode=harf;",
    "Noto Sans Cham:mode=harf;",
    "Noto Sans Saurashtra:mode=harf;",
    "Noto Sans Tamil:mode=harf;",
    "Noto Serif Tamil:mode=harf;",
    "Noto Sans Khojki:mode=harf;",
    "Noto Sans PhagsPa:mode=harf;",
    "Noto Sans Khudawadi:mode=harf;",
    "Noto Sans Deseret:mode=harf;",
    "Noto Sans Meetei Mayek:mode=harf;",
    "Noto Sans Takri:mode=harf;",
    "Noto Serif Hmong Nyiakeng:mode=harf;",
    "Noto Sans Lepcha:mode=harf;",
    "Noto Sans Old Turkic:mode=harf;",
    "Noto Sans Zanabazar Square:mode=harf;",
    "Noto Sans Osage:mode=harf;",
    "Noto Sans Sundanese:mode=harf;",
    "Noto Sans Tai Viet:mode=harf;",
    "Noto Sans Limbu:mode=harf;",
    "Noto Sans NKo:mode=harf;",
    "Noto Sans Kharoshthi:mode=harf;",
    "Noto Sans Thaana:mode=harf;",
    "Noto Sans Marchen:mode=harf;",
    "Noto Sans Lao:mode=harf;",
    "Noto Serif Lao:mode=harf;",
    "Noto Sans Gunjala Gondi:mode=harf;",
    "Noto Serif Dogra:mode=harf;",
    "Noto Sans Avestan:mode=harf;",
    "Noto Sans Mahajani:mode=harf;",
    "Noto Sans Samaritan:mode=harf;",
    "Noto Sans Tifinagh:mode=harf;",
    "Noto Sans Wancho:mode=harf;",
    "Noto Serif Ahom:mode=harf;",
    "Noto Sans Syloti Nagri:mode=harf;",
    "Noto Sans Pau Cin Hau:mode=harf;",
    "Noto Sans Hanifi Rohingya:mode=harf;",
    "Noto Sans Batak:mode=harf;",
    "Noto Sans Cypriot:mode=harf;",
    "Noto Sans Caucasian Albanian:mode=harf;",
    "Noto Sans Manichaean:mode=harf;",
    "Noto Sans Tamil Supplement:mode=harf;",
    "Noto Serif Yezidi:mode=harf;",
    "Noto Sans Tai Le:mode=harf;",
    "Noto Sans Lisu:mode=harf;",
    "Noto Sans Old Persian:mode=harf;",
    "Noto Sans Carian:mode=harf;",
    "Noto Sans Ol Chiki:mode=harf;",
    "Noto Sans Multani:mode=harf;",
    "Noto Sans Shavian:mode=harf;",
    "Noto Sans Kayah Li:mode=harf;",
    "Noto Sans Old Permic:mode=harf;",
    "Noto Sans Sogdian:mode=harf;",
    "Noto Sans Mro:mode=harf;",
    "Noto Sans Rejang:mode=harf;",
    "Noto Sans Elbasan:mode=harf;",
    "Noto Sans Nabataean:mode=harf;",
    "Noto Sans Old Sogdian:mode=harf;",
    "Noto Sans Osmanya:mode=harf;",
    "Noto Sans Old Italic:mode=harf;",
    "Noto Sans Bassa Vah:mode=harf;",
    "Noto Sans Sora Sompeng:mode=harf;",
    "Noto Sans Buginese:mode=harf;",
    "Noto Sans Mandaic:mode=harf;",
    "Noto Sans Psalter Pahlavi:mode=harf;",
    "Noto Sans Old North Arabian:mode=harf;",
    "Noto Sans Old South Arabian:mode=harf;",
    "Noto Sans Palmyrene:mode=harf;",
    "Noto Sans Imperial Aramaic:mode=harf;",
    "Noto Sans Ugaritic:mode=harf;",
    "Noto Sans Inscriptional Parthian:mode=harf;",
    "Noto Sans Gothic:mode=harf;",
    "Noto Sans Lycian:mode=harf;",
    "Noto Sans Ogham:mode=harf;",
    "Noto Sans Phoenician:mode=harf;",
    "Noto Sans Hatran:mode=harf;",
    "Noto Sans Inscriptional Pahlavi:mode=harf;",
    "Noto Sans Lydian:mode=harf;",
    "Noto Sans Elymaic:mode=harf;",
    "Noto Sans Hanunoo:mode=harf;",
    "Noto Sans Buhid:mode=harf;",
    "Noto Sans Tagalog:mode=harf;",
    "Noto Sans Tagbanwa:mode=harf;",
    "Noto Sans Mayan Numerals:mode=harf;",
    "Noto Sans Masaram Gondi:mode=harf;",
    "FreeMono:mode=harf;",
    "FreeSerif:mode=harf;",
    "FreeSans:mode=harf;",
    "DejaVu Sans:mode=harf;",
    "DejaVu Serif:mode=harf;",
    "DejaVu Sans Mono:mode=harf;",
    "WenQuanYi Zen Hei:mode=harf;",
    "Droid Sans Fallback:mode=harf;",
    "AR PL UMing CN:mode=harf;",
    "AR PL UKai CN:mode=harf;",
    "AR PL KaitiM Big5:mode=harf;",
    "AR PL Mingti2L Big5:mode=harf;",
    "IPAGothic:mode=harf;",
    "AR PL SungtiL GB:mode=harf;",
    "AR PL KaitiM GB:mode=harf;",
    "OpenSymbol:mode=harf;",
    "Chandas:mode=harf;",
    "Annapurna SIL:mode=harf;",
    "Lohit Devanagari:mode=harf;",
    "Lohit Bengali:mode=harf;",
    "Lohit Tamil:mode=harf;",
    "Lohit Gujarati:mode=harf;",
    "Lohit Gurmukhi:mode=harf;",
    "Lohit Kannada:mode=harf;",
    "Lohit Malayalam:mode=harf;",
    "Lohit Telugu:mode=harf;",
    "Lohit Odia:mode=harf;",
    "Lohit Assamese:mode=harf;",
    "Lohit Tamil Classical:mode=harf;",
    "Samyak Devanagari:mode=harf;",
    "Samyak Gujarati:mode=harf;",
    "Samyak Malayalam:mode=harf;",
    "Samyak Tamil:mode=harf;",
    "Gargi:mode=harf;",
    "Unifont:mode=harf;",
    "Unifont Upper:mode=harf;",
    "Unifont-JP:mode=harf;",
    "Unifont CSUR:mode=harf;"
  })
}
\setmonofont{Noto Sans Mono}[
  RawFeature={fallback=monofallback}
]
\setlength{\columnseprule}{0.2pt}
\setlength{\columnsep}{4pt}
\setlength{\parindent}{0pt}
\setlength{\parskip}{0pt}
\begin{document}
\pagestyle{fancy}
\thispagestyle{empty}
\null\vfill
\begin{center}
{\fontsize{36pt}{44pt}\selectfont\bfseries COVER_TITLE\par}
\end{center}
\vfill\null
\clearpage
\thispagestyle{empty}
{\rmfamily\fontsize{8pt}{10.5pt}\selectfont\setlength{\parskip}{3pt}%
{\fontsize{11pt}{14pt}\selectfont\bfseries About This Document\par}%
\medskip
This codebook was generated for use with the \textit{eight-switch} device, a USB
keyboard that encodes characters through eight binary toggle states. Each entry shows
a Unicode code point in hexadecimal, its UTF-8 encoding as four bytes in binary, and
the rendered glyph.

Character data is produced by a Rust program that iterates over all valid Unicode scalar
values (U+0000--U+10FFFF, excluding surrogate pairs). The abridged edition additionally
omits control, unassigned, private-use, and surrogate code points. Typesetting uses
LuaLaTeX with the \textit{fontspec} package and a fallback chain of over 160 fonts
to maximize glyph coverage across scripts and writing systems.
\par\medskip
{\fontsize{11pt}{14pt}\selectfont\bfseries Fonts Used\par}%
\smallskip
\textbf{Noto fonts}---Noto Sans Mono (primary), Noto Sans/Serif CJK JP, and over 150
script-specific variants. \textit{Google LLC.}\par

\textbf{HanaMinA, HanaMinB} (\texttt{花園明朝})---CJK Unified Ideographs through
Extension F. \textit{GlyphWiki\,/\,Hanazono font project.}\par

\textbf{BabelStone Han}---CJK Extension G and H. \textit{Andrew West.}\par

\textbf{FreeMono, FreeSerif, FreeSans.} \textit{GNU FreeFont contributors.}\par

\textbf{DejaVu Sans, DejaVu Serif, DejaVu Sans Mono.}
\textit{DejaVu project contributors.}\par

\textbf{WenQuanYi Zen Hei.} \textit{Wen Quan Yi project (Qianqian Fang et al.).}\par

\textbf{Droid Sans Fallback.} \textit{Ascender Corporation\,/\,Google.}\par

\textbf{AR PL UMing CN, AR PL UKai CN, AR PL KaitiM Big5, AR PL Mingti2L Big5,
AR PL SungtiL GB, AR PL KaitiM GB.} \textit{Arphic Technology Co.,~Ltd.}\par

\textbf{IPAGothic.}
\textit{Information-technology Promotion Agency (IPA), Japan.}\par

\textbf{OpenSymbol.} \textit{The Document Foundation\,/\,LibreOffice.}\par

\textbf{Chandas.} \textit{Utkal project.}\par

\textbf{Annapurna SIL.} \textit{SIL International.}\par

\textbf{Lohit Devanagari, Lohit Bengali, Lohit Tamil, Lohit Gujarati,
Lohit Gurmukhi, Lohit Kannada, Lohit Malayalam, Lohit Telugu,
Lohit Odia, Lohit Assamese, Lohit Tamil Classical.}
\textit{Red Hat,~Inc.\ and Lohit project contributors.}\par

\textbf{Samyak Devanagari, Samyak Gujarati, Samyak Malayalam, Samyak Tamil.}
\textit{DesiType.}\par

\textbf{Gargi.} \textit{CDAC (Centre for Development of Advanced Computing).}\par

\textbf{GNU Unifont, Unifont Upper, Unifont-JP, Unifont CSUR.}
\textit{Roman Czyborra, Paul Hardy, and GNU Unifont contributors.}\par
\medskip
{\fontsize{11pt}{14pt}\selectfont\bfseries With Thanks\par}%
\smallskip
The Unicode Standard is maintained by the Unicode Consortium (\texttt{unicode.org}).
Their work---encoding the world's writing systems in a single universal standard---makes
a document like this possible. Thank you to the Consortium, its member organizations,
and the researchers and script experts who have worked tirelessly to document and encode
humanity's written heritage.

Thank you also to the font designers and open-source contributors behind every font
listed above. Their collective effort---spanning decades and dozens of scripts---is what
transforms abstract code points into visible glyphs on a page.
}%
\clearpage
\raggedright
\fontsize{5.5pt}{6.5pt}\selectfont\ttfamily
\begin{multicols}{3}
{\obeylines\obeyspaces%
"#;

const POSTAMBLE: &str = r#"}%
\end{multicols}
\clearpage
\null\clearpage
\null
\end{document}
"#;

fn is_printable(ch: char) -> bool {
    !matches!(
        get_general_category(ch),
        GeneralCategory::Control
            | GeneralCategory::Unassigned
            | GeneralCategory::PrivateUse
            | GeneralCategory::Surrogate
    )
}

fn main() {
    let printable_only = std::env::args().any(|a| a == "--printable");

    let title = if printable_only {
        "UTF-8, the Whole Thing \\\\ (abridged version)"
    } else {
        "UTF-8, the Whole Thing"
    };
    let preamble = PREAMBLE_TEMPLATE.replace("COVER_TITLE", title);

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    out.write_all(preamble.as_bytes()).unwrap();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        // Input format from manual-generator:
        //   XXXXXX:BBBBBBBB BBBBBBBB BBBBBBBB BBBBBBBB:CH
        // where XXXXXX is the hex code point, BBBBBBBB... are the four UTF-8
        // bytes in binary, and CH is the actual character.
        let mut parts = line.splitn(3, ':');
        let codepoint = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        let binary = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        let ch_str = parts.next().unwrap_or("");

        let ch = match ch_str.chars().next() {
            Some(c) => c,
            None => continue,
        };

        if printable_only && !is_printable(ch) {
            continue;
        }

        // Hex digits and binary digits are LaTeX-safe as-is.
        // Only the character glyph needs escaping.
        let ch_display = escape_char(ch);

        writeln!(out, "{} {} {}", codepoint, binary, ch_display).unwrap();
    }

    out.write_all(POSTAMBLE.as_bytes()).unwrap();
}
