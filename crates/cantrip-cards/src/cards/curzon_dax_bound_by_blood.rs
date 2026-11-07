use cantrip_api::*;
use cantrip_mechanics::{choose, select, triggers};
use cantrip_oracle as oracle;

/// "Whenever another Klingon or Warrior you control dies" (CR 603.2).
fn when_another_klingon_warrior_control_dies(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::ZoneChange { old, from, to, .. } = event else { return None };
    let id = *old;
    let subject = Target::Object(id);
    if to.is(ZoneKind::Graveyard) && from.is(ZoneKind::Battlefield)
        && (!(asker.source == id)
            && ((q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Klingon))
                || q
                    .chars(id)
                    .has_subtype(q.words(asker.source).subtype(Subtype::Warrior)))
                && asker.controller == q.controller(id)))
    {
        return Fired::about(subject);
    }
    None
}

/// "Whenever another Klingon or Warrior you control dies, Curzon Dax deals 1 damage to each opponent." (CR 608.2, 120.3).
fn curzon_dax_deals_1_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let recipients_0 = {
        let q: &dyn Query = g;
        q.state()
            .opponents(item.controller)
            .collect::<Vec<PlayerId>>()
            .into_iter()
            .map(Target::Player)
            .collect::<Vec<Target>>()
    };
    let batch: Vec<(ObjectId, Target, u32)> = recipients_0
        .iter()
        .map(|t| (item.object, *t, 1u32))
        .collect();
    g.damage_together(&batch);
    Ok(())
}

/// "When Curzon Dax dies, you may put a creature card with mana value 3 or less from your hand onto the battlefield." (CR 608.2).
fn put_creature_card_mana_value(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let mut hoisted_5: Vec<ObjectId> = Vec::new();
    let mut hoisted_4: Vec<ObjectId> = Vec::new();
    let mut hoisted_3: Vec<ObjectId> = Vec::new();
    {
        let selected_1_candidates = {
            let q: &dyn Query = g;
            Some(item.controller)
                .into_iter()
                .flat_map(|p: PlayerId| q.state().zone(Zone::hand(p)).to_vec())
                .collect::<Vec<ObjectId>>()
                .into_iter()
                .filter(|&id| {
                    !q.state().obj(id).token && !q.state().obj(id).card_copy
                        && (q.chars(id).is(CardType::Creature)
                            && (i64::from(q.mana_value(id))) <= 3)
                })
                .collect::<Vec<ObjectId>>()
        };
        let selected_1 = choose::select(
            g,
            item.controller,
            item.object,
            Purpose::PutOntoBattlefield,
            selected_1_candidates,
            select::Selection::of(select::Count::UpTo(1), &[]),
        )?;
        if !selected_1.is_empty() {
            let objects_2 = selected_1
                .clone()
                .into_iter()
                .filter(|&o: &ObjectId| !g.state().obj(o).token)
                .collect::<Vec<ObjectId>>();
            let moved_2 = {
                let rows_2: Vec<(ObjectId, PlayerId, bool)> = objects_2
                    .iter()
                    .map(|&o| (o, item.controller, false))
                    .collect();
                let came_2 = g.put_all_onto_battlefield_as(&rows_2)?;
                came_2
                    .into_iter()
                    .filter(|&o| g.state().obj(o).zone.is(ZoneKind::Battlefield))
                    .collect::<Vec<ObjectId>>()
            };
            g.end_sentence();
            hoisted_3.extend(moved_2.clone());
            hoisted_4.extend(selected_1.clone());
            hoisted_5
                .extend({
                    let q: &dyn Query = g;
                    objects_2
                        .iter()
                        .copied()
                        .filter(|&o| q.state().obj(o).became.is_some())
                        .collect::<Vec<ObjectId>>()
                });
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(
                EventMask::ZONE_CHANGE,
                when_another_klingon_warrior_control_dies,
            ),
            curzon_dax_deals_1_damage,
            "Whenever another Klingon or Warrior you control dies, Curzon Dax deals 1 damage to each opponent.",
        ),
        triggered(
            triggers::THIS_DIES,
            put_creature_card_mana_value,
            "When Curzon Dax dies, you may put a creature card with mana value 3 or less from your hand onto the battlefield.",
        ),
    ],
    ..oracle::CURZON_DAX_BOUND_BY_BLOOD
};
