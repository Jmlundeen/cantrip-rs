//! Per-viewer handles for engine objects.
//!
//! A `ViewHandle` is minted the first time a viewer can observe an object and discarded the moment they cannot, so a
//! card that goes to a library and comes back gets a fresh handle and nobody can follow it through the shuffle (see
//! `Cantrip.Protocol.ViewHandle`). The engine's `ObjectId` never reaches a client. One space per viewer.
//!
//! The native engine gives an object that changes zones a new id (CR 400.7). Where the viewer saw it leave and can
//! see it arrive, the handle moves to the new object (`rebind`), so a client animating a card keeps one identity;
//! the old id stays as an alias, so a prompt that still names the old object (a trigger whose source has died) finds
//! the handle.

use std::collections::HashMap;

use cantrip_engine::ObjectId;

use crate::protocol::ViewHandle;

#[derive(Debug, Default, Clone)]
pub struct HandleSpace {
    next: u32,
    by_object: HashMap<ObjectId, ViewHandle>,
    by_handle: HashMap<ViewHandle, ObjectId>,
    /// The handles of objects that have become others (CR 400.7) and are named as they were: a question that offers them
    /// beside what they became (`former`).
    formers: HashMap<ObjectId, ViewHandle>,
}

impl HandleSpace {
    pub fn new() -> Self {
        HandleSpace { next: 1, by_object: HashMap::new(), by_handle: HashMap::new(), formers: HashMap::new() }
    }

    /// The viewer's handle for `object`, an object that has become another (CR 400.7), as it was: one of its own, naming it
    /// and not what it became, which keeps the handle it had (`rebind`). A question that offers a choice among objects (CR
    /// 609.7a: "a source of your choice") needs it, so that the object offered and the object answered are one.
    pub fn former(&mut self, object: ObjectId) -> ViewHandle {
        if let Some(&h) = self.by_object.get(&object)
            && self.by_handle.get(&h) == Some(&object)
        {
            return h;
        }
        if let Some(&h) = self.formers.get(&object) {
            return h;
        }
        let h = ViewHandle(self.next);
        self.next += 1;
        self.formers.insert(object, h);
        self.by_handle.insert(h, object);
        h
    }

    /// The handle the next object seen is given: two spaces that minted alike have the same.
    pub fn minted(&self) -> u32 {
        self.next
    }

    /// The viewer's handle for `object`, minted now if they had none.
    pub fn handle(&mut self, object: ObjectId) -> ViewHandle {
        if let Some(&h) = self.by_object.get(&object) {
            return h;
        }
        let h = ViewHandle(self.next);
        self.next += 1;
        self.by_object.insert(object, h);
        self.by_handle.insert(h, object);
        h
    }

    /// The handle the viewer already has for `object`, or none: for events about objects the viewer may hear of but
    /// may not identify.
    pub fn existing(&self, object: ObjectId) -> ViewHandle {
        self.by_object.get(&object).copied().unwrap_or(ViewHandle::NONE)
    }

    pub fn object(&self, handle: ViewHandle) -> Option<ObjectId> {
        self.by_handle.get(&handle).copied()
    }

    /// Every handle the viewer holds, with the object it names now, in handle order.
    pub fn named(&self) -> Vec<(ViewHandle, ObjectId)> {
        let mut out: Vec<(ViewHandle, ObjectId)> = self.by_handle.iter().map(|(&h, &o)| (h, o)).collect();
        out.sort_by_key(|&(h, _)| h.0);
        out
    }

    /// The viewer's handle for `old` now names `new`, the object it became.
    pub fn rebind(&mut self, old: ObjectId, new: ObjectId) {
        if let Some(&h) = self.by_object.get(&old) {
            self.by_handle.insert(h, new);
            self.by_object.insert(new, h);
        }
    }

    /// The viewer can no longer observe `object`: its handle is dropped, and the next sighting mints a new one.
    pub fn forget(&mut self, object: ObjectId) {
        // Only the handle it was offered under: the one it passed to what it became is that object's (`rebind`).
        if let Some(h) = self.formers.remove(&object) {
            self.by_handle.remove(&h);
            return;
        }
        if let Some(h) = self.by_object.remove(&object) {
            self.by_handle.remove(&h);
            self.by_object.retain(|_, v| *v != h);
        }
    }

    /// Back to the handles of an earlier position, the game having gone back to it: every handle names what it named
    /// then. A handle minted since is never minted again.
    pub fn restore(&mut self, earlier: HandleSpace) {
        let next = self.next;
        *self = earlier;
        self.next = next;
    }

    /// Drop every handle whose object `visible` refuses.
    pub fn retain(&mut self, visible: impl Fn(ObjectId) -> bool) {
        let gone: Vec<ObjectId> = self.by_handle.values().copied().filter(|o| !visible(*o)).collect();
        for o in gone {
            self.forget(o);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An object that became another keeps its alias on the handle of what it became, and is offered as it was under a handle of
    /// its own: answering that handle names the former object, not the one it became; it is the same handle each time asked,
    /// and goes when the object may no longer be seen.
    #[test]
    fn a_former_object_has_a_handle_of_its_own() {
        let (old, new) = (ObjectId(3), ObjectId(9));
        let mut handles = HandleSpace::new();
        let seen = handles.handle(old);
        handles.rebind(old, new);
        assert_eq!(handles.object(seen), Some(new), "the handle follows the card");
        assert_eq!(handles.handle(old), seen, "the old id stays an alias");
        let former = handles.former(old);
        assert_ne!(former, seen);
        assert_eq!(handles.object(former), Some(old));
        assert_eq!(handles.former(old), former);
        assert_eq!(handles.former(new), seen, "an object that has not become another has its own handle");
        handles.retain(|o| o == new);
        assert_eq!(handles.object(former), None);
        assert_eq!(handles.object(seen), Some(new));
    }
}
