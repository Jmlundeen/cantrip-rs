use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "The next spell you cast this turn can't be countered" (CR 611.2c, 601.2, 701.5a): an effect about the ability's
/// controller, which marks the next spell that player casts this turn as it is cast, before anyone has priority, and ends
/// with the turn if they cast none. Each activation is its own effect, so two before the next spell both apply to it.
fn next_spell_cant_be_countered(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_player_effect(item.controller, item.object, EffectKind::NextSpellCantBeCountered, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Mountain) }, { sub(Subtype::Forest) }>),
    abilities: &[
        mana::tap_for(mana::add_u, Mana::U, "{T}: Add {U}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{U}, {T}: The next spell you cast this turn can't be countered.",
            cost: Cost { mana: mana(0, 0, 1, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: next_spell_cant_be_countered,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::MISTRISE_VILLAGE
};
