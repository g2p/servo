/**
NativeFont encapsulates access to the platform's font API,
e.g. quartz, FreeType. It provides access to metrics and tables
needed by the text shaper as well as access to the underlying
font resources needed by the graphics layer to draw glyphs.
*/

use glyph::GlyphIndex;
use font_cache::native::NativeFontCache;

#[cfg(target_os = "macos")]
pub type NativeFont/& = quartz_native_font::QuartzNativeFont;

#[cfg(target_os = "linux")]
pub type NativeFont/& = ft_native_font::FreeTypeNativeFont;

#[cfg(target_os = "macos")]
pub fn create(_native_lib: &NativeFontCache, buf: @~[u8]) -> Result<NativeFont, ()> {
    quartz_native_font::create(buf)
}

#[cfg(target_os = "linux")]
pub fn create(native_lib: &NativeFontCache, buf: @~[u8]) -> Result<NativeFont, ()> {
    ft_native_font::create(native_lib, buf)
}

#[cfg(target_os = "macos")]
pub fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    quartz_native_font::with_test_native_font(f);
}

#[cfg(target_os = "linux")]
pub fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    ft_native_font::with_test_native_font(f);
}

#[test]
fn should_get_glyph_indexes() {
    with_test_native_font(|font| {
        let idx = font.glyph_index('w');
        assert idx == Some(40u as GlyphIndex);
    })
}

#[test]
fn should_return_none_glyph_index_for_bad_codepoints() {
    with_test_native_font(|font| {
        let idx = font.glyph_index(0 as char);
        assert idx == None;
    })
}

#[test]
#[ignore]
fn should_get_glyph_h_advance() {
    with_test_native_font(|font| {
        let adv = font.glyph_h_advance(40u as GlyphIndex);
        // TODO: add correct advances; these are old
        assert adv == Some(15f);
    })
}

#[test]
#[ignore]
fn should_return_none_glyph_h_advance_for_bad_codepoints() {
    with_test_native_font(|font| {
        let adv = font.glyph_h_advance(-1 as GlyphIndex);
        assert adv == None;
    })
}
