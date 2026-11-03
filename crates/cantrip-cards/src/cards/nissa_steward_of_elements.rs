use cantrip_api::*;
use cantrip_mechanics::{Animation, animate, effects, targets, top_of_library};
use cantrip_oracle as oracle;

const PUT_ONTO_THE_BATTLEFIELD: Question = Question::new(
    "PutTopCardOntoBattlefield",
    "put the top card of your library onto the battlefield?",
    "It is a land card or a creature card with mana value no greater than the loyalty counters on Nissa",
    "Put it onto the battlefield",
    "Leave it on top",
);

/// "0: Look at the top card of your library. If it's a land card or a creature card with mana value less than or equal to the
/// number of loyalty counters on Nissa, you may put that card onto the battlefield." (CR 608.2c, 122.1, 202.3).
fn look_top_card_library_if(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let top = top_of_library(g, item.controller, 1);
    g.look_at(item.controller, &top);
    g.end_sentence();
    let Some(&card) = top.first() else { return Ok(()) };
    let loyalty = g.obj(item.object).counters(CounterKind::Loyalty);
    let eligible = g.chars(card).is(CardType::Land) || g.chars(card).is(CardType::Creature) && i32::from(loyalty) >= g.mana_value(card) as i32;
    if eligible && effects::ask_may(g, item.controller, item.object, Purpose::Ask(&PUT_ONTO_THE_BATTLEFIELD))? {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

/// "−6: Untap up to two target lands you control. They become 5/5 Elemental creatures with flying and haste until end of turn.
/// They're still lands." (CR 701.26a, 613.1d, 613.1f, 613.4b): the Elemental type is the one Nissa's own text has now (CR 612).
fn untap_up_two_target_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let lands = targets::legal_objects(g, item);
    effects::untap_targets(g, item)?;
    g.end_sentence();
    let elemental = g.words(item.object).subtype(Subtype::Elemental);
    let animation = Animation {
        power: 5,
        toughness: 5,
        colors: None,
        types: TypeSet::NONE,
        subtypes: &[elemental],
        all_creature_types: false,
        replaces_creature_types: false,
        keywords: Keywords::of(&[Keyword::Flying, Keyword::Haste]),
        ability: None,
    };
    // "They become ..." is one instruction for both lands (CR 611.2c): the state triggers judge them animated together.
    g.one_instruction(&mut |g| {
        for &land in &lands {
            animate(g, item.object, land, &animation, Duration::EndOfTurn);
        }
        Ok(())
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(2), |g, item| effects::scry(g, item, 2), "+2: Scry 2."),
        ability(
            loyalty(0),
            look_top_card_library_if,
            "0: Look at the top card of your library. If it's a land card or a creature card with mana value less than or equal to the number of loyalty counters on Nissa, you may put that card onto the battlefield.",
        ),
        ability(
            loyalty(-6),
            untap_up_two_target_lands,
            "−6: Untap up to two target lands you control. They become 5/5 Elemental creatures with flying and haste until end of turn. They're still lands.",
        )
        .targets(&[targets::LAND_YOU_CONTROL.between(0, 2)]),
    ],
    enters: Some(effects::enters_with_x_counters::<{ CounterKind::Loyalty as usize }>),
    ..oracle::NISSA_STEWARD_OF_ELEMENTS
};
