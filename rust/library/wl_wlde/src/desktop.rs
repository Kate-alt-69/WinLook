//! Portable desktop item grouping, adaptive application-icon metadata and context-menu contracts.
//!
//! This module never moves, renames, hides or deletes Shell items. A DesktopGroup
//! is only a reference collection for a future platform presenter. Native
//! presentation is still gated by PresentationCapabilities::groups_available().

use std::collections::HashSet;

use crate::PresentationCapabilities;

pub const MAX_DESKTOP_GROUPS: usize = 64;
pub const MAX_DESKTOP_GROUP_MEMBERS: usize = 128;
pub const MAX_DESKTOP_IDENTIFIER_BYTES: usize = 127;
pub const MAX_DESKTOP_LABEL_BYTES: usize = 255;
pub const MAX_ICON_ASSET_REFERENCE_BYTES: usize = 255;
pub const DEFAULT_ADAPTIVE_ICON_SAFE_ZONE_PERCENT: u8 = 61;

/// An item discovered by the platform adapter. Its ID is an opaque adapter key,
/// not a path; the portable core never opens or changes the underlying item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopItemKind {
    File,
    Folder,
    Application,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DesktopItemId(String);

impl DesktopItemId {
    pub fn new(value: impl Into<String>) -> Result<Self, DesktopModelError> {
        let value = value.into();
        validate_text(
            &value,
            MAX_DESKTOP_IDENTIFIER_BYTES,
            DesktopModelError::EmptyIdentifier,
            DesktopModelError::IdentifierTooLong,
        )?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopItemRef {
    id: DesktopItemId,
    label: String,
    kind: DesktopItemKind,
}

impl DesktopItemRef {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: DesktopItemKind,
    ) -> Result<Self, DesktopModelError> {
        let id = DesktopItemId::new(id)?;
        let label = label.into();
        validate_text(
            &label,
            MAX_DESKTOP_LABEL_BYTES,
            DesktopModelError::EmptyLabel,
            DesktopModelError::LabelTooLong,
        )?;
        Ok(Self { id, label, kind })
    }

    pub fn id(&self) -> &DesktopItemId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn kind(&self) -> DesktopItemKind {
        self.kind
    }
}

/// A virtual collection. Members continue to exist at their original Shell or
/// filesystem locations; grouping does not create a directory or duplicate data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopGroup {
    id: String,
    label: String,
    members: Vec<DesktopItemId>,
}

impl DesktopGroup {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        members: Vec<DesktopItemId>,
    ) -> Result<Self, DesktopModelError> {
        let id = id.into();
        validate_text(
            &id,
            MAX_DESKTOP_IDENTIFIER_BYTES,
            DesktopModelError::EmptyIdentifier,
            DesktopModelError::IdentifierTooLong,
        )?;
        let label = label.into();
        validate_text(
            &label,
            MAX_DESKTOP_LABEL_BYTES,
            DesktopModelError::EmptyLabel,
            DesktopModelError::LabelTooLong,
        )?;
        if members.is_empty() {
            return Err(DesktopModelError::EmptyMembers);
        }
        if members.len() > MAX_DESKTOP_GROUP_MEMBERS {
            return Err(DesktopModelError::TooManyMembers);
        }
        let mut unique_members = HashSet::with_capacity(members.len());
        if members.iter().any(|member| !unique_members.insert(member)) {
            return Err(DesktopModelError::DuplicateMember);
        }
        Ok(Self { id, label, members })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn members(&self) -> &[DesktopItemId] {
        &self.members
    }
}

/// Validated local-only group configuration. Replacements are transactional:
/// an invalid update never destroys the last known-good collection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesktopGroups {
    groups: Vec<DesktopGroup>,
}

impl DesktopGroups {
    pub fn new(
        groups: Vec<DesktopGroup>,
        available_items: &[DesktopItemRef],
    ) -> Result<Self, DesktopModelError> {
        validate_groups(&groups, available_items)?;
        Ok(Self { groups })
    }

    pub fn replace(
        &mut self,
        groups: Vec<DesktopGroup>,
        available_items: &[DesktopItemRef],
    ) -> Result<(), DesktopModelError> {
        validate_groups(&groups, available_items)?;
        self.groups = groups;
        Ok(())
    }

    /// Adds a new virtual group after validating every referenced item.
    /// Invalid replacements leave the last known-good group set unchanged.
    pub fn add_group(
        &mut self,
        group: DesktopGroup,
        available_items: &[DesktopItemRef],
    ) -> Result<(), DesktopModelError> {
        let mut replacement = self.groups.clone();
        replacement.push(group);
        self.replace(replacement, available_items)
    }

    /// Renames a group without changing its member references.
    pub fn rename_group(
        &mut self,
        group_id: &str,
        label: impl Into<String>,
        available_items: &[DesktopItemRef],
    ) -> Result<(), DesktopModelError> {
        let index = self.group_index(group_id)?;
        let current = &self.groups[index];
        let renamed = DesktopGroup::new(current.id.clone(), label, current.members.clone())?;
        let mut replacement = self.groups.clone();
        replacement[index] = renamed;
        self.replace(replacement, available_items)
    }

    /// Adds members to a group. Duplicate or stale item IDs are rejected
    /// before the active collection is replaced.
    pub fn add_members(
        &mut self,
        group_id: &str,
        members: impl IntoIterator<Item = DesktopItemId>,
        available_items: &[DesktopItemRef],
    ) -> Result<(), DesktopModelError> {
        let index = self.group_index(group_id)?;
        let current = &self.groups[index];
        let mut combined = current.members.clone();
        combined.extend(members);
        let updated = DesktopGroup::new(
            current.id.clone(),
            current.label.clone(),
            combined,
        )?;
        let mut replacement = self.groups.clone();
        replacement[index] = updated;
        self.replace(replacement, available_items)
    }

    /// Removes members from a group. Removing its final member removes the
    /// now-empty Group record; underlying Shell items remain untouched.
    pub fn remove_members(
        &mut self,
        group_id: &str,
        members: &[DesktopItemId],
        available_items: &[DesktopItemRef],
    ) -> Result<(), DesktopModelError> {
        let index = self.group_index(group_id)?;
        if members.is_empty() {
            return Ok(());
        }

        let mut requested = HashSet::with_capacity(members.len());
        if members.iter().any(|member| !requested.insert(member.clone())) {
            return Err(DesktopModelError::DuplicateMember);
        }

        let current = &self.groups[index];
        if members.iter().any(|member| !current.members.contains(member)) {
            return Err(DesktopModelError::MemberNotInGroup);
        }

        let remaining = current
            .members
            .iter()
            .filter(|member| !requested.contains(*member))
            .cloned()
            .collect::<Vec<_>>();
        let mut replacement = self.groups.clone();
        if remaining.is_empty() {
            replacement.remove(index);
        } else {
            replacement[index] = DesktopGroup::new(
                current.id.clone(),
                current.label.clone(),
                remaining,
            )?;
        }
        self.replace(replacement, available_items)
    }

    /// Removes only the virtual Group record. It never removes its Shell items.
    pub fn ungroup(
        &mut self,
        group_id: &str,
        available_items: &[DesktopItemRef],
    ) -> Result<(), DesktopModelError> {
        let index = self.group_index(group_id)?;
        let mut replacement = self.groups.clone();
        replacement.remove(index);
        self.replace(replacement, available_items)
    }

    /// Looks up a group by its stable ID.
    pub fn get(&self, group_id: &str) -> Option<&DesktopGroup> {
        self.groups.iter().find(|group| group.id() == group_id)
    }

    /// Checks membership and reports an unknown Group explicitly.
    pub fn is_member(
        &self,
        group_id: &str,
        item_id: &DesktopItemId,
    ) -> Result<bool, DesktopModelError> {
        let group = self.get(group_id).ok_or(DesktopModelError::UnknownGroup)?;
        Ok(group.members().contains(item_id))
    }

    fn group_index(&self, group_id: &str) -> Result<usize, DesktopModelError> {
        self.groups
            .iter()
            .position(|group| group.id() == group_id)
            .ok_or(DesktopModelError::UnknownGroup)
    }

    pub fn groups(&self) -> &[DesktopGroup] {
        &self.groups
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Configured groups are renderable only when the adapter can also restore
    /// the original native items after a crash, disable or uninstall.
    pub fn can_present(&self, capabilities: &PresentationCapabilities) -> bool {
        !self.groups.is_empty() && capabilities.groups_available()
    }
}

fn validate_groups(
    groups: &[DesktopGroup],
    available_items: &[DesktopItemRef],
) -> Result<(), DesktopModelError> {
    if groups.len() > MAX_DESKTOP_GROUPS {
        return Err(DesktopModelError::TooManyGroups);
    }

    let mut available = HashSet::with_capacity(available_items.len());
    for item in available_items {
        if !available.insert(item.id()) {
            return Err(DesktopModelError::DuplicateItemIdentifier);
        }
    }
    let mut group_ids = HashSet::with_capacity(groups.len());

    for group in groups {
        if !group_ids.insert(group.id.as_str()) {
            return Err(DesktopModelError::DuplicateIdentifier);
        }
        if group.members.is_empty() {
            return Err(DesktopModelError::EmptyMembers);
        }
        if group.members.len() > MAX_DESKTOP_GROUP_MEMBERS {
            return Err(DesktopModelError::TooManyMembers);
        }
        let mut member_ids = HashSet::with_capacity(group.members.len());
        for member in &group.members {
            if !available.contains(member) {
                return Err(DesktopModelError::UnknownMember);
            }
            if !member_ids.insert(member) {
                return Err(DesktopModelError::DuplicateMember);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppIconMask {
    Circle,
    RoundedSquare,
    Squircle,
}

/// Separate asset references mirror the foreground/background structure of
/// adaptive launcher icons. The platform adapter resolves and decodes these
/// references; WLDE composes the supplied RGBA layers in adaptive_icon.rs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppIconLayers {
    foreground: String,
    background: String,
    monochrome: Option<String>,
}

impl AppIconLayers {
    pub fn new(
        foreground: impl Into<String>,
        background: impl Into<String>,
        monochrome: Option<String>,
    ) -> Result<Self, DesktopModelError> {
        let foreground = foreground.into();
        let background = background.into();
        validate_icon_asset(&foreground)?;
        validate_icon_asset(&background)?;
        if let Some(asset) = monochrome.as_deref() {
            validate_icon_asset(asset)?;
        }
        Ok(Self {
            foreground,
            background,
            monochrome,
        })
    }

    pub fn foreground(&self) -> &str {
        &self.foreground
    }

    pub fn background(&self) -> &str {
        &self.background
    }

    pub fn monochrome(&self) -> Option<&str> {
        self.monochrome.as_deref()
    }
}

/// WLDE's normalized application tile description. The selected mask and
/// safe-zone affect presentation only; original executable, shortcut and icon
/// source files are not rewritten.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdaptiveAppIcon {
    application_id: String,
    label: String,
    layers: AppIconLayers,
    mask: AppIconMask,
    safe_zone_percent: u8,
}

impl AdaptiveAppIcon {
    pub fn new(
        application_id: impl Into<String>,
        label: impl Into<String>,
        layers: AppIconLayers,
        mask: AppIconMask,
        safe_zone_percent: u8,
    ) -> Result<Self, DesktopModelError> {
        let application_id = application_id.into();
        validate_text(
            &application_id,
            MAX_DESKTOP_IDENTIFIER_BYTES,
            DesktopModelError::EmptyIdentifier,
            DesktopModelError::IdentifierTooLong,
        )?;
        let label = label.into();
        validate_text(
            &label,
            MAX_DESKTOP_LABEL_BYTES,
            DesktopModelError::EmptyLabel,
            DesktopModelError::LabelTooLong,
        )?;
        if !(48..=72).contains(&safe_zone_percent) {
            return Err(DesktopModelError::InvalidSafeZone);
        }
        Ok(Self {
            application_id,
            label,
            layers,
            mask,
            safe_zone_percent,
        })
    }

    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn layers(&self) -> &AppIconLayers {
        &self.layers
    }

    pub fn mask(&self) -> AppIconMask {
        self.mask
    }

    pub fn safe_zone_percent(&self) -> u8 {
        self.safe_zone_percent
    }
}

fn validate_icon_asset(value: &str) -> Result<(), DesktopModelError> {
    validate_text(
        value,
        MAX_ICON_ASSET_REFERENCE_BYTES,
        DesktopModelError::EmptyAssetReference,
        DesktopModelError::AssetReferenceTooLong,
    )
}

fn validate_text(
    value: &str,
    max_bytes: usize,
    empty_error: DesktopModelError,
    too_long_error: DesktopModelError,
) -> Result<(), DesktopModelError> {
    if value.trim().is_empty() || value.contains('\0') {
        return Err(empty_error);
    }
    if value.len() > max_bytes {
        return Err(too_long_error);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopContextTarget {
    File,
    Folder,
    Application,
    Group,
    DesktopBackground,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopContextAction {
    Open,
    AddToGroup,
    RemoveFromGroup,
    CustomizeApplicationIcon,
    OpenGroup,
    AddGroupMembers,
    RenameGroup,
    Ungroup,
    Properties,
    CreateGroup,
    OpenDesktopSettings,
}

const FILE_CONTEXT_ACTIONS: &[DesktopContextAction] = &[
    DesktopContextAction::Open,
    DesktopContextAction::AddToGroup,
    DesktopContextAction::Properties,
];
const GROUPED_FILE_CONTEXT_ACTIONS: &[DesktopContextAction] = &[
    DesktopContextAction::Open,
    DesktopContextAction::AddToGroup,
    DesktopContextAction::RemoveFromGroup,
    DesktopContextAction::Properties,
];
const APPLICATION_CONTEXT_ACTIONS: &[DesktopContextAction] = &[
    DesktopContextAction::Open,
    DesktopContextAction::AddToGroup,
    DesktopContextAction::CustomizeApplicationIcon,
    DesktopContextAction::Properties,
];
const GROUPED_APPLICATION_CONTEXT_ACTIONS: &[DesktopContextAction] = &[
    DesktopContextAction::Open,
    DesktopContextAction::AddToGroup,
    DesktopContextAction::RemoveFromGroup,
    DesktopContextAction::CustomizeApplicationIcon,
    DesktopContextAction::Properties,
];
const GROUP_CONTEXT_ACTIONS: &[DesktopContextAction] = &[
    DesktopContextAction::OpenGroup,
    DesktopContextAction::AddGroupMembers,
    DesktopContextAction::RenameGroup,
    DesktopContextAction::Ungroup,
    DesktopContextAction::Properties,
];
const DESKTOP_BACKGROUND_CONTEXT_ACTIONS: &[DesktopContextAction] = &[
    DesktopContextAction::CreateGroup,
    DesktopContextAction::OpenDesktopSettings,
];

/// Returns WLDE's type-aware right-click action contract. The Shell adapter
/// remains responsible for invoking native verbs and preserving OS behaviour.
pub fn desktop_context_actions(
    target: DesktopContextTarget,
    is_group_member: bool,
) -> &'static [DesktopContextAction] {
    match target {
        DesktopContextTarget::File | DesktopContextTarget::Folder => {
            if is_group_member {
                GROUPED_FILE_CONTEXT_ACTIONS
            } else {
                FILE_CONTEXT_ACTIONS
            }
        }
        DesktopContextTarget::Application => {
            if is_group_member {
                GROUPED_APPLICATION_CONTEXT_ACTIONS
            } else {
                APPLICATION_CONTEXT_ACTIONS
            }
        }
        DesktopContextTarget::Group => GROUP_CONTEXT_ACTIONS,
        DesktopContextTarget::DesktopBackground => DESKTOP_BACKGROUND_CONTEXT_ACTIONS,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopModelError {
    EmptyIdentifier,
    IdentifierTooLong,
    EmptyLabel,
    LabelTooLong,
    EmptyMembers,
    TooManyMembers,
    TooManyGroups,
    DuplicateIdentifier,
    DuplicateItemIdentifier,
    DuplicateMember,
    UnknownMember,
    EmptyAssetReference,
    AssetReferenceTooLong,
    InvalidSafeZone,
    UnknownGroup,
    MemberNotInGroup,
}

impl std::fmt::Display for DesktopModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier => f.write_str("desktop identifiers cannot be empty"),
            Self::IdentifierTooLong => f.write_str("desktop identifier exceeds the portable limit"),
            Self::EmptyLabel => f.write_str("desktop labels cannot be empty"),
            Self::LabelTooLong => f.write_str("desktop label exceeds the portable limit"),
            Self::EmptyMembers => f.write_str("a desktop group must contain at least one item"),
            Self::TooManyMembers => f.write_str("desktop group exceeds the 128-member limit"),
            Self::TooManyGroups => f.write_str("desktop group count exceeds the 64-group limit"),
            Self::DuplicateIdentifier => f.write_str("desktop group identifiers must be unique"),
            Self::DuplicateItemIdentifier => {
                f.write_str("desktop inventory item identifiers must be unique")
            }
            Self::DuplicateMember => f.write_str("a desktop group cannot list the same item twice"),
            Self::UnknownMember => {
                f.write_str("desktop group references an item outside the current inventory")
            }
            Self::EmptyAssetReference => {
                f.write_str("adaptive icon asset references cannot be empty")
            }
            Self::AssetReferenceTooLong => {
                f.write_str("adaptive icon asset reference exceeds the portable limit")
            }
            Self::InvalidSafeZone => {
                f.write_str("adaptive icon safe zone must be between 48 and 72 percent")
            }
            Self::UnknownGroup => f.write_str("desktop Group does not exist"),
            Self::MemberNotInGroup => f.write_str("desktop item is not a member of this Group"),
        }
    }
}

impl std::error::Error for DesktopModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn desktop_items() -> Vec<DesktopItemRef> {
        vec![
            DesktopItemRef::new("app:editor", "Editor", DesktopItemKind::Application).unwrap(),
            DesktopItemRef::new("file:notes", "Notes.txt", DesktopItemKind::File).unwrap(),
            DesktopItemRef::new("folder:assets", "Assets", DesktopItemKind::Folder).unwrap(),
        ]
    }

    #[test]
    fn groups_are_virtual_bounded_and_reference_existing_items() {
        let items = desktop_items();
        let group = DesktopGroup::new(
            "group:work",
            "Work",
            vec![
                DesktopItemId::new("app:editor").unwrap(),
                DesktopItemId::new("file:notes").unwrap(),
                DesktopItemId::new("folder:assets").unwrap(),
            ],
        )
        .unwrap();
        let groups = DesktopGroups::new(vec![group], &items).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups.groups()[0].members().len(), 3);

        let mut capabilities = PresentationCapabilities {
            virtual_groups: true,
            ..PresentationCapabilities::default()
        };
        assert!(!groups.can_present(&capabilities));
        capabilities.crash_safe_group_restore = true;
        assert!(groups.can_present(&capabilities));
    }

    #[test]
    fn group_replacement_is_transactional_and_rejects_unknown_items() {
        let items = desktop_items();
        let initial = DesktopGroup::new(
            "group:work",
            "Work",
            vec![DesktopItemId::new("app:editor").unwrap()],
        )
        .unwrap();
        let mut groups = DesktopGroups::new(vec![initial], &items).unwrap();

        let invalid = DesktopGroup::new(
            "group:invalid",
            "Invalid",
            vec![DesktopItemId::new("file:missing").unwrap()],
        )
        .unwrap();
        assert_eq!(
            groups.replace(vec![invalid], &items),
            Err(DesktopModelError::UnknownMember)
        );
        assert_eq!(groups.groups()[0].id(), "group:work");
    }

    #[test]
    fn desktop_inventory_ids_must_be_unique() {
        let items = desktop_items();
        let duplicate_inventory = vec![items[0].clone(), items[0].clone()];
        let group = DesktopGroup::new(
            "group:work",
            "Work",
            vec![DesktopItemId::new("app:editor").unwrap()],
        )
        .unwrap();
        assert_eq!(
            DesktopGroups::new(vec![group], &duplicate_inventory),
            Err(DesktopModelError::DuplicateItemIdentifier)
        );
    }

    #[test]
    fn group_members_and_group_ids_must_be_unique() {
        let items = desktop_items();
        let duplicate_members = DesktopGroup::new(
            "group:duplicate",
            "Duplicate",
            vec![
                DesktopItemId::new("app:editor").unwrap(),
                DesktopItemId::new("app:editor").unwrap(),
            ],
        );
        assert_eq!(duplicate_members, Err(DesktopModelError::DuplicateMember));

        let first = DesktopGroup::new(
            "group:shared",
            "First",
            vec![DesktopItemId::new("app:editor").unwrap()],
        )
        .unwrap();
        let second = DesktopGroup::new(
            "group:shared",
            "Second",
            vec![DesktopItemId::new("file:notes").unwrap()],
        )
        .unwrap();
        assert_eq!(
            DesktopGroups::new(vec![first, second], &items),
            Err(DesktopModelError::DuplicateIdentifier)
        );
    }

    #[test]
    fn group_commands_create_rename_add_remove_and_ungroup_transactionally() {
        let items = desktop_items();
        let mut groups = DesktopGroups::default();
        let group = DesktopGroup::new(
            "group:work",
            "Work",
            vec![DesktopItemId::new("app:editor").unwrap()],
        )
        .unwrap();

        groups.add_group(group, &items).unwrap();
        assert_eq!(groups.get("group:work").unwrap().label(), "Work");
        groups.rename_group("group:work", "Projects", &items).unwrap();
        assert_eq!(groups.get("group:work").unwrap().label(), "Projects");

        let file_id = DesktopItemId::new("file:notes").unwrap();
        groups.add_members("group:work", [file_id.clone()], &items).unwrap();
        assert!(groups.is_member("group:work", &file_id).unwrap());

        let before_duplicate = groups.clone();
        assert_eq!(
            groups.add_members("group:work", [file_id.clone()], &items),
            Err(DesktopModelError::DuplicateMember)
        );
        assert_eq!(groups, before_duplicate);

        groups.remove_members("group:work", &[file_id], &items).unwrap();
        let app_id = DesktopItemId::new("app:editor").unwrap();
        groups.remove_members("group:work", &[app_id], &items).unwrap();
        assert!(groups.is_empty());

        groups.add_group(
            DesktopGroup::new(
                "group:temporary",
                "Temporary",
                vec![DesktopItemId::new("folder:assets").unwrap()],
            ).unwrap(),
            &items,
        ).unwrap();
        groups.ungroup("group:temporary", &items).unwrap();
        assert!(groups.is_empty());
        assert_eq!(groups.ungroup("group:missing", &items), Err(DesktopModelError::UnknownGroup));
    }

    #[test]
    fn group_mutations_reject_missing_members_without_losing_previous_state() {
        let items = desktop_items();
        let mut groups = DesktopGroups::new(
            vec![DesktopGroup::new(
                "group:work",
                "Work",
                vec![DesktopItemId::new("app:editor").unwrap()],
            ).unwrap()],
            &items,
        ).unwrap();
        let before = groups.clone();

        assert_eq!(
            groups.remove_members(
                "group:work",
                &[DesktopItemId::new("file:notes").unwrap()],
                &items,
            ),
            Err(DesktopModelError::MemberNotInGroup)
        );
        assert_eq!(groups, before);
        assert_eq!(
            groups.rename_group("group:missing", "New name", &items),
            Err(DesktopModelError::UnknownGroup)
        );
        assert_eq!(groups, before);
    }

    #[test]
    fn adaptive_app_icons_keep_separate_layers_and_validate_safe_zone() {
        let layers = AppIconLayers::new(
            "asset:editor-foreground",
            "asset:editor-background",
            Some("asset:editor-mono".to_string()),
        )
        .unwrap();
        let icon = AdaptiveAppIcon::new(
            "app:editor",
            "Editor",
            layers,
            AppIconMask::Squircle,
            DEFAULT_ADAPTIVE_ICON_SAFE_ZONE_PERCENT,
        )
        .unwrap();
        assert_eq!(icon.mask(), AppIconMask::Squircle);
        assert_eq!(icon.layers().foreground(), "asset:editor-foreground");
        assert_eq!(icon.layers().background(), "asset:editor-background");
        assert_eq!(icon.layers().monochrome(), Some("asset:editor-mono"));
        assert_eq!(
            AdaptiveAppIcon::new(
                "app:editor",
                "Editor",
                icon.layers().clone(),
                AppIconMask::Circle,
                90,
            ),
            Err(DesktopModelError::InvalidSafeZone)
        );
    }

    #[test]
    fn right_click_actions_are_type_aware_and_non_destructive() {
        let app_actions = desktop_context_actions(DesktopContextTarget::Application, true);
        assert!(app_actions.contains(&DesktopContextAction::CustomizeApplicationIcon));
        assert!(app_actions.contains(&DesktopContextAction::RemoveFromGroup));
        assert!(!app_actions.contains(&DesktopContextAction::Ungroup));

        let file_actions = desktop_context_actions(DesktopContextTarget::File, false);
        assert!(file_actions.contains(&DesktopContextAction::AddToGroup));
        assert!(!file_actions.contains(&DesktopContextAction::CustomizeApplicationIcon));
        assert!(!file_actions.contains(&DesktopContextAction::Ungroup));

        let group_actions = desktop_context_actions(DesktopContextTarget::Group, false);
        assert!(group_actions.contains(&DesktopContextAction::Ungroup));
        assert!(group_actions.contains(&DesktopContextAction::RenameGroup));

        let background_actions =
            desktop_context_actions(DesktopContextTarget::DesktopBackground, false);
        assert!(background_actions.contains(&DesktopContextAction::CreateGroup));
        assert!(background_actions.contains(&DesktopContextAction::OpenDesktopSettings));
    }
}
