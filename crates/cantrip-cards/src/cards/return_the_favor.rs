use cantrip_api::*;
use cantrip_mechanics::{effects, spree, targets};
use cantrip_oracle as oracle;

/// "Target instant spell, sorcery spell, activated ability, or triggered ability" (CR 115.1): an instant or sorcery spell on
/// the stack, of the types it has there, or any ability on the stack (a mana ability never is, CR 605.3b).
fn instant_sorcery_or_ability(q: &dyn Query, _: &Asker, t: Target) -> bool {
    match t {
        Target::Ability(serial) => q.state().stack.iter().any(|i| i.kind != StackKind::Spell && i.serial == serial),
        Target::Object(id) => {
            let c = q.chars(id);
            q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id) && (c.is(CardType::Instant) || c.is(CardType::Sorcery))
        }
        _ => false,
    }
}

/// "Copy target instant spell, sorcery spell, activated ability, or triggered ability. You may choose new targets for the
/// copy." (CR 707.10, 707.10c) A spell's copy is not cast; an ability's is neither activated nor triggered.
fn copy_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    match item.target(0) {
        Target::Object(spell) => {
            g.copy_spell(spell, item.controller, true)?;
        }
        ability => {
            g.copy_ability(ability, item.controller, true)?;
        }
    }
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(copy_target, "+ {1} — Copy target instant spell, sorcery spell, activated ability, or triggered ability. You may choose new targets for the copy.")
        .targets(&[target(instant_sorcery_or_ability)])
        .cost(mana("{1}")),
    mode(effects::change_first_target, "+ {1} — Change the target of target spell or ability with a single target.")
        .targets(&[targets::SPELL_OR_ABILITY_WITH_A_SINGLE_TARGET])
        .cost(mana("{1}")),
];

static MODAL: ModalDef = spree::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::RETURN_THE_FAVOR };
