//! Every card's characteristics (docs/engine-design.md, "Card data: three sources"): generated at build time from content/ (build.rs), never
//! written by hand. A card's behavior adds to its constant here: `CardDef { spell: ..., ..cantrip_oracle::LIGHTNING_BOLT }`.

#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use cantrip_api::{Color, ColorSet};

    #[test]
    fn commander_identities_include_every_face_and_ignore_reminder_symbols() {
        assert_eq!(KENRITH_THE_RETURNED_KING.color_identity, ColorSet(31));
        assert_eq!(ROGRAKH_SON_OF_ROHGAHH.color_identity, ColorSet::of(Color::Red));
        assert_eq!(VALKI_GOD_OF_LIES.color_identity, ColorSet::of(Color::Black).with(Color::Red));
        assert_eq!(BASILICA_GUARDS.color_identity, ColorSet::of(Color::White), "extort's reminder-text hybrid symbol does not add black");
        assert_eq!(SPHINX_OF_THE_GUILDPACT.color_identity, ColorSet(31), "a color-defining ability contributes identity");
        assert_eq!(ELUSIVE_OTTER.color_identity, ColorSet::of(Color::Blue).with(Color::Green), "the adventure's green cost contributes identity");
        assert_eq!(BLITZ_AUTOMATON.color_identity, ColorSet::of(Color::Red), "prototype's alternative red cost contributes identity");
    }
}
