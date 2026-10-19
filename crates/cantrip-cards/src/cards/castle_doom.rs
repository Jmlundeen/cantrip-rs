use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Create a 3/3 colorless Robot Villain artifact creature token named Doombot." The artifact was sacrificed as a cost.
fn create_doombot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::DOOMBOT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_any_color(1).only_for(ManaRestriction::ARTIFACT_SPELLS), "{T}: Add one mana of any color. Spend this mana only to cast an artifact spell."),
        ability(cost("{3}, {T}").and(sacrifice(1, targets::filters::artifact)), create_doombot, "{3}, {T}, Sacrifice an artifact: Create a 3/3 colorless Robot Villain artifact creature token named Doombot. Activate only as a sorcery.")
            .sorcery_speed(),
    ],
    ..oracle::CASTLE_DOOM
};

pub static EXTRA: &[&CardDef] = &[&tokens::DOOMBOT];
