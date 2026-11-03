use cantrip_api::*;
use cantrip_mechanics::{prevention, select};
use cantrip_oracle as oracle;

/// "{1}: The next time a black source of your choice would deal damage to you this turn, prevent that damage." (CR 615.1).
fn made_2_applies(
    q: &dyn Query,
    asker: &Asker,
    affected: ObjectId,
    e: &DamageEvent,
) -> bool {
    {
        let source = e.source;
        if prevention::by_chosen_source(q, affected, e)
            && q.chars(source).colors.has(q.words(asker.source).color(Color::Black))
        {
            return true;
        }
    }
    false
}

/// "{1}: The next time a black source of your choice would deal damage to you this turn, prevent that damage." (CR 608.2, 611.2a).
fn black_source(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |id| q.chars(id).colors.has(q.words(choice.source).color(Color::Black)))
}

fn next_time_black_source_choice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::Selection::new(select::Count::AsManyAsCan(1))
        .with(select::part(&(black_source as SelectFn), 0, "black"))
        .called("source of damage");
    if let Some(source) = prevention::choose_source(g, item, rule)? {
        g.add_effect(item.object, source, EffectKind::Prevention {
            card: item.ability.card, index: 0, to: Target::Player(item.controller), shield: Shield::NextTime,
        }, Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}"),
            next_time_black_source_choice,
            "{1}: The next time a black source of your choice would deal damage to you this turn, prevent that damage.",
        ),
    ],
    granted_preventions: &[
        GrantedPreventionDef {
            hint: "{1}: The next time a black source of your choice would deal damage to you this turn, prevent that damage.",
            damage: "damage",
            of_it: None,
            applies: made_2_applies,
            then: None,
            text: "The next time a black source of your choice would deal damage to you this turn, prevent that damage.",
        },
    ],
    ..oracle::CIRCLE_OF_PROTECTION_BLACK
};
