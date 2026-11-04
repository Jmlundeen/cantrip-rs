use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::triggers::EQUIPPED_CREATURE_ATTACKS_SUBJECT;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose up to one target creature. That creature can't be blocked this turn and has base power and toughness 1/1 until end of
/// turn" (CR 509.1b, 613.4b, 611.2): both on the one target, nothing if none was chosen or it is no longer legal (CR 608.2b).
fn unblockable_one_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    // One sentence about one creature (CR 608.2c): the state triggers see both changes done together.
    g.one_instruction(&mut |g| {
        effects::target_creature_cant_be_blocked_this_turn(g, item)?;
        if let Some(creature) = targets::object(g, item, 0) {
            g.add_effect(item.object, creature, EffectKind::SetPt(1, 1), Duration::EndOfTurn);
        }
        Ok(())
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            EQUIPPED_CREATURE_ATTACKS_SUBJECT,
            unblockable_one_one,
            "Whenever equipped creature attacks, choose up to one target creature. That creature can't be blocked this turn and has base power and toughness 1/1 until end of turn.",
        )
        .targets(&[up_to_one(targets::creature_filter)]),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<1, 0>, "Equipped creature gets +1/+0.")],
    ..oracle::ATOMIC_MICROSIZER
};
