use cantrip_api::*;
use cantrip_mechanics::{triggers, destroy_target, targets};
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

fn becomes_monstrous(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Monstrous { permanent } if *permanent == source => Fired::PLAIN,
        _ => None,
    }
}

fn monstrosity_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.monstrosity(item.object, 4);
    Ok(())
}

const TEXT: &str = "When this creature enters or becomes monstrous, destroy target permanent.";

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "Enters or becomes monstrous" is two triggers with the same effect.
        triggered(triggers::THIS_ENTERS, destroy_it, TEXT)
            .targets(&[targets::PERMANENT]),
        triggered(when(EventMask::MONSTROUS, becomes_monstrous), destroy_it, TEXT)
            .targets(&[targets::PERMANENT]),
        ability(cost("{5}{B}{G}"), monstrosity_four, "{5}{B}{G}: Monstrosity 4. (If this creature isn't monstrous, put four +1/+1 counters on it and it becomes monstrous.)"),
    ],
    ..oracle::ALPHA_DEATHCLAW
};
