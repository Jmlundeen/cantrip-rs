use cantrip_api::*;
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

/// "If you control a commander as you cast this spell" (CR 903.3, 601.2b, 700.2a): a commander permanent on the
/// battlefield under the caster's control. A commander in the command zone is not controlled, and a copy of one is no
/// commander (CR 707.2); `permanents_of` is the battlefield alone.
fn controls_a_commander(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().any(|id| q.obj(id).commander)
}

/// "Create a 1/1 colorless Borg artifact creature token for each attacking creature." (CR 111.1, 506.3, 608.2c): every
/// creature attacking any player, planeswalker or battle, counted as the mode resolves.
fn create_borg_for_each_attacker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let attackers = g.state().attackers.len() as u32;
    tokens::create(g, item.controller, &tokens::BORG, attackers)?;
    Ok(())
}

/// "Draw a card for each attacking creature." (CR 121.1, 506.3, 608.2c): counted as the mode resolves.
fn draw_for_each_attacker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let attackers = g.state().attackers.len() as u32;
    effects::draw(g, item, attackers)
}

/// "Choose one. If you control a commander as you cast this spell, you may choose both instead." (CR 601.2b, 700.2):
/// one mode normally, either or both while the condition holds as the modes are chosen. The modes run in printed order
/// (CR 700.2d).
static MODAL: ModalDef = modes(&[
    mode(create_borg_for_each_attacker, "Create a 1/1 colorless Borg artifact creature token for each attacking creature."),
    mode(draw_for_each_attacker, "Draw a card for each attacking creature."),
])
.up_to_if(controls_a_commander, 2);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::COLLECTIVE_OVERRIDE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
