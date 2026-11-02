use cantrip_api::*;
use cantrip_mechanics::prevention;
use cantrip_oracle as oracle;

/// "Prevent all combat damage that would be dealt this turn. Each attacking creature doesn't untap during its controller's
/// next untap step." (CR 615.1, 502.3): the attacking creatures are the ones attacking as Tangle resolves, each under its
/// own effect until the next untap step of whoever controls it then (the Frost Titan and Tamiyo, the Moon Sage rulings).
fn prevent_and_keep_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::all_combat_damage_this_turn(g, item, 0);
    g.end_sentence();
    let attacking: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.is_attacking(id) && g.chars(id).is(CardType::Creature)).collect();
    instruction(g, |g| {
        for &id in &attacking {
            g.add_effect(item.object, id, EffectKind::DoesntUntap, Duration::DuringItsControllersNextUntapStep);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], prevent_and_keep_tapped),
    granted_preventions: &[prevention::COMBAT_DAMAGE.text("Prevent all combat damage that would be dealt this turn.")],
    ..oracle::TANGLE
};
