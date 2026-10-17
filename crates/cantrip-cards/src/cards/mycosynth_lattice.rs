use cantrip_api::*;
use cantrip_oracle as oracle;

/// "All permanents" (CR 110.1): every permanent on the battlefield, Mycosynth Lattice itself among them. Spells and cards
/// elsewhere are no permanents, so they don't become artifacts (the ruling).
fn all_permanents(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, _: &Chars) -> bool {
    true
}

/// "... are artifacts in addition to their other types" (CR 205.1b, 613.1d, layer 4). An Aura made an artifact stays an
/// Aura; one that also becomes a creature (March of the Machines) can't enchant anything (CR 303.4d, the ruling).
fn artifact_too(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Artifact);
}

/// "All cards that aren't on the battlefield, spells, and permanents": every object in every zone (the ruling: "everything,
/// in every zone of the game").
fn everything(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, _: &Chars) -> bool {
    true
}

/// "... are colorless" (CR 105.2c, 613.1e, layer 5).
fn colorless(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.colors = ColorSet::NONE;
}

/// "Players may spend mana as though it were mana of any color." (CR 609.4b): every player, for every payment - a spell, an
/// ability, any other mana cost - and colorless mana too (the ruling). It changes no cost, and no restriction on the mana
/// (the ruling: mana that may be spent only on artifact spells still may not pay for another spell).
fn any_payment(_: &dyn Query, _: &Asker, _: &Spending) -> bool {
    true
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Type, all_permanents, artifact_too, "All permanents are artifacts in addition to their other types."),
        static_ability(Layer::Color, everything, colorless, "All cards that aren't on the battlefield, spells, and permanents are colorless.")
            .everywhere(),
    ],
    spend_any_color: &[any_payment],
    ..oracle::MYCOSYNTH_LATTICE
};
