use cantrip_api::*;
use cantrip_mechanics::{each_opponent_sacrifices, effects, select, tokens};
use cantrip_oracle as oracle;

/// I: "Each opponent sacrifices a nontoken creature of their choice" (CR 701.21a, 101.4). An opponent with no nontoken
/// creature sacrifices nothing.
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, |q, id| {
        q.chars(id).is(CardType::Creature) && !q.obj(id).token
    })
}

/// II: "Investigate" (CR 701.16a): a Clue token under this Saga's controller.
fn chapter_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::investigate(g, item.controller)
}

/// III: "You may sacrifice an artifact or creature. If you do, create two 2/2 black Romulan creature tokens" (CR 608.2c,
/// 701.21a). Nothing is asked and nothing is created when there is nothing to sacrifice.
fn chapter_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::artifact_or_creature_permanents(select::up_to(1));
    if !effects::may_sacrifice(g, item.controller, item.object, rule)?.is_empty() {
        tokens::create(g, item.controller, &tokens::ROMULAN, 2)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, chapter_one, "I \u{2014} Each opponent sacrifices a nontoken creature of their choice."),
        chapter(2, chapter_two, "II \u{2014} Investigate."),
        chapter(
            3,
            chapter_three,
            "III \u{2014} You may sacrifice an artifact or creature. If you do, create two 2/2 black Romulan creature tokens.",
        ),
    ],
    ..oracle::IN_THE_PALE_MOONLIGHT
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROMULAN];
