use cantrip_api::*;
use cantrip_mechanics::mobilize;
use cantrip_oracle as oracle;

/// "Your opponents can't cast spells during your turn" (CR 101.2): a cast rule, true for who may cast.
fn opponents_cant_cast_on_your_turn(q: &dyn Query, asker: &Asker, caster: PlayerId) -> bool {
    !(q.state().turn.active == asker.controller && caster != asker.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[mobilize::trigger::<2>("Mobilize 2 (Whenever this creature attacks, create two tapped and attacking 1/1 red Warrior creature tokens. Sacrifice them at the beginning of the next end step.)")],
    delayed: &[mobilize::SACRIFICE],
    cast_rules: &[opponents_cant_cast_on_your_turn],
    ..oracle::VOICE_OF_VICTORY
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
