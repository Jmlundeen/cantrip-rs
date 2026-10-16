use cantrip_api::*;
use cantrip_mechanics::statics::creature_dying_triggers_again;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

/// "Put an indestructible counter on Drivnod." (CR 122.1b): nothing once it has left the battlefield (CR 400.7).
fn indestructible_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Indestructible, 1);
    Ok(())
}

/// "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time." (CR 603.2d)
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{B/P}{B/P}").and(exile_from_graveyard_of(3, filters::creature)),
        indestructible_counter,
        "{B/P}{B/P}, Exile three creature cards from your graveyard: Put an indestructible counter on Drivnod. ({B/P} can be paid with either {B} or 2 life.)",
    )],
    additional_triggers: &[creature_dying_triggers_again],
    ..oracle::DRIVNOD_CARNAGE_DOMINUS
};
