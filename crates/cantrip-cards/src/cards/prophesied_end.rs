use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Destroy target creature. If it wasn't attacking, its controller draws a card." (CR 701.8, 121.1): whether it was
/// attacking (CR 506.3) and who controlled it are read before the creature is destroyed (CR 608.2h), and the draw
/// follows if it was not attacking.
fn prophesy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let attacking = g.is_attacking(creature);
    let controller = g.controller(creature);
    g.destroy_all(&[creature]);
    if !attacking {
        g.draw(controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], prophesy),
    ..oracle::PROPHESIED_END
};
