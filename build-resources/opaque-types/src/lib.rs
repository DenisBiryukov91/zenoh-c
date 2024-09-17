#![allow(unused_doc_comments)]
#![allow(dead_code)]
use core::ffi::c_void;
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
use std::sync::Arc;
use std::{
    sync::{Condvar, Mutex, MutexGuard},
    thread::JoinHandle,
};

use zenoh::{
    bytes::{Encoding, ZBytes, ZBytesIterator, ZBytesReader, ZBytesSliceIterator, ZBytesWriter},
    config::Config,
    handlers::RingChannelHandler,
    key_expr::KeyExpr,
    pubsub::{Publisher, Subscriber},
    query::{Query, Queryable, Reply, ReplyError},
    sample::Sample,
    scouting::Hello,
    session::Session,
    time::Timestamp,
};
#[cfg(feature = "unstable")]
use zenoh::{
    liveliness::LivelinessToken,
    pubsub::MatchingListener,
    sample::SourceInfo,
    session::{EntityGlobalId, ZenohId},
};
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
use zenoh::{
    shm::zshm, shm::zshmmut, shm::AllocLayout, shm::ChunkAllocResult, shm::ChunkDescriptor,
    shm::DynamicProtocolID, shm::MemoryLayout, shm::PosixShmProviderBackend, shm::ProtocolID,
    shm::ShmClient, shm::ShmClientStorage, shm::ShmProvider, shm::ShmProviderBackend,
    shm::StaticProtocolID, shm::ZLayoutError, shm::ZShm, shm::ZShmMut, shm::POSIX_PROTOCOL_ID,
};

#[macro_export]
macro_rules! get_opaque_type_data {
    ($src_type:ty, $name:ident) => {
        const _: () = {
            use const_format::concatcp;
            const DST_NAME: &str = stringify!($name);
            const ALIGN: usize = std::mem::align_of::<$src_type>();
            const SIZE: usize = std::mem::size_of::<$src_type>();
            const INFO_MESSAGE: &str =
                concatcp!("type: ", DST_NAME, ", align: ", ALIGN, ", size: ", SIZE);
            #[cfg(feature = "panic")]
            panic!("{}", INFO_MESSAGE);
        };
    };
}

/// A serialized Zenoh data.
///
/// To minimize copies and reallocations, Zenoh may provide data in several separate buffers.
get_opaque_type_data!(ZBytes, z_owned_bytes_t);
/// A loaned serialized Zenoh data.
get_opaque_type_data!(ZBytes, z_loaned_bytes_t);

pub struct CSlice {
    _data: *const u8,
    _len: usize,
    _drop: Option<extern "C" fn(data: *mut c_void, context: *mut c_void)>,
    _context: *mut c_void,
}

get_opaque_type_data!(CSlice, z_owned_slice_t);
/// A contiguous sequence of bytes owned by some other entity.
get_opaque_type_data!(CSlice, z_view_slice_t);
/// A loaned sequence of bytes.
get_opaque_type_data!(CSlice, z_loaned_slice_t);

/// The wrapper type for strings allocated by Zenoh.
get_opaque_type_data!(CSlice, z_owned_string_t);
/// The view over a string.
get_opaque_type_data!(CSlice, z_view_string_t);
/// A loaned string.
get_opaque_type_data!(CSlice, z_loaned_string_t);

/// An array of maybe-owned non-null terminated strings.
///
get_opaque_type_data!(Vec<CSlice>, z_owned_string_array_t);
/// A loaned string array.
get_opaque_type_data!(Vec<CSlice>, z_loaned_string_array_t);

/// An owned Zenoh sample.
///
/// This is a read only type that can only be constructed by cloning a `z_loaned_sample_t`.
/// Like all owned types, it should be freed using z_drop or z_sample_drop.
get_opaque_type_data!(Option<Sample>, z_owned_sample_t);
/// A loaned Zenoh sample.
get_opaque_type_data!(Sample, z_loaned_sample_t);

/// A reader for serialized data.
get_opaque_type_data!(ZBytesReader<'static>, z_bytes_reader_t);

/// A writer for serialized data.
get_opaque_type_data!(ZBytesWriter<'static>, z_bytes_writer_t);

/// An iterator over multi-element serialized data.
get_opaque_type_data!(ZBytesIterator<'static, ZBytes>, z_bytes_iterator_t);

/// An iterator over slices of serialized data.
get_opaque_type_data!(ZBytesSliceIterator<'static>, z_bytes_slice_iterator_t);

/// The <a href="https://zenoh.io/docs/manual/abstractions/#encoding"> encoding </a> of Zenoh data.
get_opaque_type_data!(Encoding, z_owned_encoding_t);
/// A loaned Zenoh encoding.
get_opaque_type_data!(Encoding, z_loaned_encoding_t);

/// An owned reply from a Queryable to a `z_get()`.
get_opaque_type_data!(Option<Reply>, z_owned_reply_t);
/// A loaned reply.
get_opaque_type_data!(Reply, z_loaned_reply_t);

/// A Zenoh reply error - a combination of reply error payload and its encoding.
get_opaque_type_data!(ReplyError, z_owned_reply_err_t);
/// A loaned Zenoh reply error.
get_opaque_type_data!(ReplyError, z_loaned_reply_err_t);

/// An owned Zenoh query received by a queryable.
///
/// Queries are atomically reference-counted, letting you extract them from the callback that handed them to you by cloning.
get_opaque_type_data!(Option<Query>, z_owned_query_t);
/// A loaned Zenoh query.
get_opaque_type_data!(Query, z_loaned_query_t);

/// An owned Zenoh <a href="https://zenoh.io/docs/manual/abstractions/#queryable"> queryable </a>.
///
/// Responds to queries sent via `z_get()` with intersecting key expression.
get_opaque_type_data!(Option<Queryable<()>>, z_owned_queryable_t);
/// A loaned Zenoh queryable.
get_opaque_type_data!(Queryable<()>, z_loaned_queryable_t);
#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned Zenoh querying subscriber.
///
/// In addition to receiving the data it is subscribed to,
/// it also will fetch data from a Queryable at startup and peridodically (using  `ze_querying_subscriber_get()`).
get_opaque_type_data!(
    Option<(zenoh_ext::FetchingSubscriber<()>, &'static Session)>,
    ze_owned_querying_subscriber_t
);
#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned Zenoh querying subscriber.
get_opaque_type_data!(
    (zenoh_ext::FetchingSubscriber<()>, &'static Session),
    ze_loaned_querying_subscriber_t
);

/// A Zenoh-allocated <a href="https://zenoh.io/docs/manual/abstractions/#key-expression"> key expression </a>.
///
/// Key expressions can identify a single key or a set of keys.
///
/// Examples :
///    - ``"key/expression"``.
///    - ``"key/ex*"``.
///
/// Key expressions can be mapped to numerical ids through `z_declare_keyexpr`
/// for wire and computation efficiency.
///
/// Internally key expressiobn can be either:
///   - A plain string expression.
///   - A pure numerical id.
///   - The combination of a numerical prefix and a string suffix.
get_opaque_type_data!(Option<KeyExpr<'static>>, z_owned_keyexpr_t);
/// A user allocated string, viewed as a key expression.
get_opaque_type_data!(Option<KeyExpr<'static>>, z_view_keyexpr_t);

/// A loaned key expression.
///
/// Key expressions can identify a single key or a set of keys.
///
/// Examples :
///    - ``"key/expression"``.
///    - ``"key/ex*"``.
///
/// Using `z_declare_keyexpr` allows Zenoh to optimize a key expression,
/// both for local processing and network-wise.
get_opaque_type_data!(KeyExpr<'static>, z_loaned_keyexpr_t);

/// An owned Zenoh session.
get_opaque_type_data!(Option<Session>, z_owned_session_t);
/// A loaned Zenoh session.
get_opaque_type_data!(Session, z_loaned_session_t);

/// An owned Zenoh configuration.
get_opaque_type_data!(Option<Config>, z_owned_config_t);
/// A loaned Zenoh configuration.
get_opaque_type_data!(Config, z_loaned_config_t);

#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A Zenoh ID.
///
/// In general, valid Zenoh IDs are LSB-first 128bit unsigned and non-zero integers.
get_opaque_type_data!(ZenohId, z_id_t);

/// A Zenoh <a href="https://zenoh.io/docs/manual/abstractions/#timestamp"> timestamp </a>.
///
/// It consists of a time generated by a Hybrid Logical Clock (HLC) in NPT64 format and a unique zenoh identifier.
get_opaque_type_data!(Timestamp, z_timestamp_t);

/// An owned Zenoh <a href="https://zenoh.io/docs/manual/abstractions/#publisher"> publisher </a>.
get_opaque_type_data!(Option<Publisher<'static>>, z_owned_publisher_t);
/// A loaned Zenoh publisher.
get_opaque_type_data!(Publisher<'static>, z_loaned_publisher_t);

#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned Zenoh matching listener.
///
/// A listener that sends notifications when the [`MatchingStatus`] of a publisher changes.
/// Dropping the corresponding publisher, also drops matching listener.
get_opaque_type_data!(Option<MatchingListener<()>>, zc_owned_matching_listener_t);

/// An owned Zenoh <a href="https://zenoh.io/docs/manual/abstractions/#subscriber"> subscriber </a>.
///
/// Receives data from publication on intersecting key expressions.
/// Destroying the subscriber cancels the subscription.
get_opaque_type_data!(Option<Subscriber<()>>, z_owned_subscriber_t);
/// A loaned Zenoh subscriber.
get_opaque_type_data!(Subscriber<()>, z_loaned_subscriber_t);

#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A liveliness token that can be used to provide the network with information about connectivity to its
/// declarer: when constructed, a PUT sample will be received by liveliness subscribers on intersecting key
/// expressions.
///
/// A DELETE on the token's key expression will be received by subscribers if the token is destroyed, or if connectivity between the subscriber and the token's creator is lost.
get_opaque_type_data!(Option<LivelinessToken>, zc_owned_liveliness_token_t);
#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
get_opaque_type_data!(LivelinessToken, zc_loaned_liveliness_token_t);
#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned Zenoh publication cache.
///
/// Used to store publications on intersecting key expressions. Can be queried later via `z_get()` to retrieve this data
/// (for example by `ze_owned_querying_subscriber_t`).
get_opaque_type_data!(
    Option<zenoh_ext::PublicationCache>,
    ze_owned_publication_cache_t
);
#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned Zenoh publication cache.
get_opaque_type_data!(zenoh_ext::PublicationCache, ze_loaned_publication_cache_t);

/// An owned mutex.
get_opaque_type_data!(
    Option<(Mutex<()>, Option<MutexGuard<'static, ()>>)>,
    z_owned_mutex_t
);
/// A loaned mutex.
get_opaque_type_data!(
    (Mutex<()>, Option<MutexGuard<'static, ()>>),
    z_loaned_mutex_t
);

/// An owned conditional variable.
///
/// Used in combination with `z_owned_mutex_t` to wake up thread when certain conditions are met.
get_opaque_type_data!(Option<Condvar>, z_owned_condvar_t);
/// A loaned conditional variable.
get_opaque_type_data!(Condvar, z_loaned_condvar_t);

/// An owned Zenoh task.
get_opaque_type_data!(Option<JoinHandle<()>>, z_owned_task_t);

/// An owned Zenoh-allocated hello message returned by a Zenoh entity to a scout message sent with `z_scout()`.
get_opaque_type_data!(Option<Hello>, z_owned_hello_t);
/// A loaned hello message.
get_opaque_type_data!(Hello, z_loaned_hello_t);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned SHM Client.
get_opaque_type_data!(Option<Arc<dyn ShmClient>>, z_owned_shm_client_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned list of SHM Clients.
get_opaque_type_data!(
    Option<Vec<(ProtocolID, Arc<dyn ShmClient>)>>,
    zc_owned_shm_client_list_t
);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned list of SHM Clients.
get_opaque_type_data!(
    Vec<(ProtocolID, Arc<dyn ShmClient>)>,
    zc_loaned_shm_client_list_t
);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned SHM Client Storage
get_opaque_type_data!(Option<Arc<ShmClientStorage>>, z_owned_shm_client_storage_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// A loaned SHM Client Storage.
get_opaque_type_data!(Arc<ShmClientStorage>, z_loaned_shm_client_storage_t);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned MemoryLayout.
get_opaque_type_data!(Option<MemoryLayout>, z_owned_memory_layout_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned MemoryLayout.
get_opaque_type_data!(MemoryLayout, z_loaned_memory_layout_t);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned ChunkAllocResult.
get_opaque_type_data!(Option<ChunkAllocResult>, z_owned_chunk_alloc_result_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned ChunkAllocResult.
get_opaque_type_data!(ChunkAllocResult, z_loaned_chunk_alloc_result_t);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned ZShm slice.
get_opaque_type_data!(Option<ZShm>, z_owned_shm_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned ZShm slice.
get_opaque_type_data!(zshm, z_loaned_shm_t);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned ZShmMut slice.
get_opaque_type_data!(Option<ZShmMut>, z_owned_shm_mut_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned ZShmMut slice.
get_opaque_type_data!(zshmmut, z_loaned_shm_mut_t);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
#[derive(Debug)]
#[repr(C)]
struct DummyCallbacks {
    alloc_fn: unsafe extern "C" fn(),
    free_fn: unsafe extern "C" fn(),
    defragment_fn: unsafe extern "C" fn() -> usize,
    available_fn: unsafe extern "C" fn() -> usize,
    layout_for_fn: unsafe extern "C" fn(),
}

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
#[derive(Debug)]
#[repr(C)]
struct DummyContext {
    context: *mut c_void,
    delete_fn: unsafe extern "C" fn(*mut c_void),
}

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
#[derive(Debug)]
struct DummySHMProviderBackend {
    context: DummyContext,
    callbacks: DummyCallbacks,
}

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
impl ShmProviderBackend for DummySHMProviderBackend {
    fn alloc(&self, _layout: &MemoryLayout) -> ChunkAllocResult {
        todo!()
    }

    fn free(&self, _chunk: &ChunkDescriptor) {
        todo!()
    }

    fn defragment(&self) -> usize {
        todo!()
    }

    fn available(&self) -> usize {
        todo!()
    }

    fn layout_for(&self, _layout: MemoryLayout) -> Result<MemoryLayout, ZLayoutError> {
        todo!()
    }
}

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
type DummySHMProvider = ShmProvider<DynamicProtocolID, DummySHMProviderBackend>;

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
type PosixSHMProvider = ShmProvider<StaticProtocolID<POSIX_PROTOCOL_ID>, PosixShmProviderBackend>;

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
enum CDummySHMProvider {
    Posix(PosixSHMProvider),
    Dynamic(DummySHMProvider),
}

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned ShmProvider.
get_opaque_type_data!(Option<CDummySHMProvider>, z_owned_shm_provider_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned ShmProvider.
get_opaque_type_data!(CDummySHMProvider, z_loaned_shm_provider_t);

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
type PosixAllocLayout =
    AllocLayout<'static, StaticProtocolID<POSIX_PROTOCOL_ID>, PosixShmProviderBackend>;

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
type DummyDynamicAllocLayout = AllocLayout<'static, DynamicProtocolID, DummySHMProviderBackend>;

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
enum CSHMLayout {
    Posix(PosixAllocLayout),
    Dynamic(DummyDynamicAllocLayout),
}

#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned ShmProvider's AllocLayout.
get_opaque_type_data!(Option<CSHMLayout>, z_owned_alloc_layout_t);
#[cfg(all(feature = "shared-memory", feature = "unstable"))]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned ShmProvider's AllocLayout.
get_opaque_type_data!(CSHMLayout, z_loaned_alloc_layout_t);

/// An owned Zenoh fifo sample handler.
get_opaque_type_data!(
    Option<flume::Receiver<Sample>>,
    z_owned_fifo_handler_sample_t
);
/// An loaned Zenoh fifo sample handler.
get_opaque_type_data!(flume::Receiver<Sample>, z_loaned_fifo_handler_sample_t);

/// An owned Zenoh ring sample handler.
get_opaque_type_data!(
    Option<RingChannelHandler<Sample>>,
    z_owned_ring_handler_sample_t
);
/// An loaned Zenoh ring sample handler.
get_opaque_type_data!(RingChannelHandler<Sample>, z_loaned_ring_handler_sample_t);

/// An owned Zenoh fifo query handler.
get_opaque_type_data!(Option<flume::Receiver<Query>>, z_owned_fifo_handler_query_t);
/// An loaned Zenoh fifo query handler.
get_opaque_type_data!(flume::Receiver<Query>, z_loaned_fifo_handler_query_t);

/// An owned Zenoh ring query handler.
get_opaque_type_data!(
    Option<RingChannelHandler<Query>>,
    z_owned_ring_handler_query_t
);
/// An loaned Zenoh ring query handler.
get_opaque_type_data!(RingChannelHandler<Query>, z_loaned_ring_handler_query_t);

/// An owned Zenoh fifo reply handler.
get_opaque_type_data!(Option<flume::Receiver<Reply>>, z_owned_fifo_handler_reply_t);
/// An loaned Zenoh fifo reply handler.
get_opaque_type_data!(flume::Receiver<Reply>, z_loaned_fifo_handler_reply_t);

/// An owned Zenoh ring reply handler.
get_opaque_type_data!(
    Option<RingChannelHandler<Reply>>,
    z_owned_ring_handler_reply_t
);
/// An loaned Zenoh ring reply handler.
get_opaque_type_data!(RingChannelHandler<Reply>, z_loaned_ring_handler_reply_t);

#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An owned Zenoh-allocated source info`.
get_opaque_type_data!(SourceInfo, z_owned_source_info_t);
#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief A loaned source info.
get_opaque_type_data!(SourceInfo, z_loaned_source_info_t);
#[cfg(feature = "unstable")]
/// @warning This API has been marked as unstable: it works as advertised, but it may be changed in a future release.
/// @brief An entity gloabal id.
get_opaque_type_data!(EntityGlobalId, z_entity_global_id_t);