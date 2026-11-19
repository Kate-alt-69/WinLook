//! Traton is WinLook's version-aware Windows object and provider layer.
//!
//! The portable core parses the namespace, detects/represents Windows targets,
//! validates typed object/member requests, tracks capability evidence, selects
//! compatible provider descriptors and transports bounded normalized events.
//! Win32 calls are isolated in the Windows platform adapter.

#![deny(unsafe_op_in_unsafe_fn)]

mod capability;
mod events;
mod namespace;
mod objects;
mod provider;
mod registry;
mod target;

#[cfg(windows)]
mod windows;

pub use capability::{CapabilityError, CapabilityRecord, CapabilitySet, CapabilityStatus};
pub use events::{
    BoundedEventQueue, EventQueueError, EventQueuePushResult, TratonEvent, TratonEventKind,
    TratonEventPayload, DEFAULT_EVENT_QUEUE_CAPACITY, MAX_EVENT_QUEUE_CAPACITY,
};
pub use namespace::{NamespacePath, NamespacePathError};
pub use objects::{ObjectReference, ObjectReferenceError, ObjectReferenceKind};
pub use provider::{
    AppliedState, OperationResult, OperationStatus, ProviderCatalog, ProviderDescriptor,
    ProviderError, ProviderKind, RestoreSnapshot, TranslationProvider, MAX_RESTORE_PAYLOAD_BYTES,
};
pub use registry::{
    ArgumentPolicy, MemberDefinition, ObjectDefinition, ObjectKind, ObjectOwnership,
    ObjectRegistry, OperationAction, PropertyValue, RegistryError, ValidatedOperation, ValueSchema,
};
pub use target::{
    Architecture, TargetDetectionError, WindowsFamily, WindowsTarget, WindowsTargetInfo,
};
