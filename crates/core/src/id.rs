// ID types for TrueWorld

use serde::{Deserialize, Serialize};
use std::fmt;

/// Entity ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// Player ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(u64);

impl PlayerId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player({})", self.0)
    }
}

/// Room ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(u64);

impl RoomId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Room({})", self.0)
    }
}

/// Skill ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId(u32);

impl SkillId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Skill({})", self.0)
    }
}

/// Item ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(u32);

impl ItemId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item({})", self.0)
    }
}

/// Monster ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonsterId(u32);

impl MonsterId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for MonsterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Monster({})", self.0)
    }
}

/// Server ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerId(u64);

impl ServerId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server({})", self.0)
    }
}

/// Pet ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PetId(u64);

impl PetId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for PetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pet({})", self.0)
    }
}

/// Object ID (for world objects like items, interactive objects)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(u32);

impl ObjectId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object({})", self.0)
    }
}

/// Attack Type ID (for different attack types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttackTypeId(u16);

impl AttackTypeId {
    pub fn new(id: u16) -> Self {
        Self(id)
    }

    pub fn raw(self) -> u16 {
        self.0
    }
}

impl fmt::Display for AttackTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AttackType({})", self.0)
    }
}
