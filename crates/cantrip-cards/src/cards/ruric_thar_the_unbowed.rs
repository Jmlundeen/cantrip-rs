use cantrip_api::*;
use cantrip_mechanics::{statics, triggers};
use cantrip_oracle as oracle;

/// "Ruric Thar attacks each combat if able." (CR 604.1).
fn ruric_thar_attacks_combat_if_apply_0(
    _: &dyn LayerQuery,
    _: ObjectId,
    chars: &mut Chars,
) {
    chars.must_attack += 1;
}

/// "Whenever a player casts a noncreature spell, Ruric Thar deals 6 damage to that player." (CR 608.2, 120.3): the trigger is
/// about the player who cast it (`item.subject`).
fn ruric_thar_deals_6_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(player) = item.subject else { return Ok(()) };
    g.damage(item.object, Target::Player(player), 6);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::A_PLAYER_CASTS_A_NONCREATURE_SPELL,
            ruric_thar_deals_6_damage,
            "Whenever a player casts a noncreature spell, Ruric Thar deals 6 damage to that player.",
        ),
    ],
    statics: &[
        static_ability(
            Layer::Ability,
            statics::itself,
            ruric_thar_attacks_combat_if_apply_0,
            "Ruric Thar attacks each combat if able.",
        ),
    ],
    ..oracle::RURIC_THAR_THE_UNBOWED
};
