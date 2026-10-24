use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast an instant or sorcery spell, put a page counter on this artifact" (CR 603.2, 122.1).
fn page(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Page, 1);
    Ok(())
}

/// "This ability costs {1} less to activate for each page counter on this artifact" (CR 601.2f, 602.2b): generic mana only,
/// never below zero of it ({5} has five generic to give).
fn cheaper_per_page_counter(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    let source = asker.source;
    CostDelta::less(q.state().obj(source).counters(CounterKind::Page).clamp(0, 255) as u8)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::INSTANT_OR_SORCERY_YOU_CAST,
            page,
            "Whenever you cast an instant or sorcery spell, put a page counter on this artifact.",
        ),
        ability(
            cost("{5}, {T}"),
            |g, item| effects::draw(g, item, 1),
            "{5}, {T}: Draw a card. This ability costs {1} less to activate for each page counter on this artifact.",
        )
        .cost_change(cheaper_per_page_counter),
    ],
    ..oracle::DIARY_OF_DREAMS
};
