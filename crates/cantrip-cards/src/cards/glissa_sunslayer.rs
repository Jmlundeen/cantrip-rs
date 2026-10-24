use cantrip_api::*;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Remove up to three counters from target permanent": its controller chooses which, of any kinds, as it resolves (CR 608.2d).
fn remove_mode(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    targets::object(g, item, 0).map(|permanent| g.remove_counters_up_to(item.controller, item.object, permanent, 3)).transpose()?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::draw_a_card_and_lose_1_life, "You draw a card and lose 1 life."),
    mode(effects::destroy_first_target, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
    mode(remove_mode, "Remove up to three counters from target permanent.").targets(&[targets::PERMANENT]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, &MODAL, "Whenever this creature deals combat damage to a player, choose one \u{2014} You draw a card and lose 1 life; or destroy target enchantment; or remove up to three counters from target permanent.")],
    ..oracle::GLISSA_SUNSLAYER
};
