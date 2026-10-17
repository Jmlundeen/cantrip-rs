use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create a Heartwood token."
fn soul_tether(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_heartwood(g, item.controller)
}

/// Heartwood Crafter's prepare spell (CR 722): not a card, only ever a copy, cast from exile (CR 722.3c).
pub static SOUL_TETHER: CardDef = CardDef {
    name: "Soul Tether",
    cost: mana("{2}{R/G}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], soul_tether),
    text: "Create a Heartwood token. (It's a red and green artifact with \"{T}: Add {R} or {G}.\")",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add(Mana::C).only_for(ManaRestriction::NOT_SPELLS_FROM_HAND),
        "{T}: Add {C}. This mana can't be spent to cast spells from your hand.",
    )],
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&SOUL_TETHER),
    ..oracle::HEARTWOOD_CRAFTER
};

pub static EXTRA: &[&CardDef] = &[&SOUL_TETHER, &tokens::HEARTWOOD];
