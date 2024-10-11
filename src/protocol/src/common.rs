
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonReplay {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ClusterType {
    PlacementCenter = 0,
    JournalServer = 1,
    MqttBrokerServer = 2,
    AmqpBrokerServer = 3,
}

impl ClusterType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ClusterType::PlacementCenter => "PlacementCenter",
            ClusterType::JournalServer => "JournalServer",
            ClusterType::MqttBrokerServer => "MQTTBrokerServer",
            ClusterType::AmqpBrokerServer => "AMQPBrokerServer",
        }
    }

    pub fn from_st_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PlacementCenter" => Some(Self::PlacementCenter),
            "JournalServer" => Some(Self::JournalServer),
            "MQTTBrokerServer" => Some(Self::MqttBrokerServer),
            "AMQPBrokerServer" => Some(Self::AmqpBrokerServer),
            _ => None,
        }
    }
}