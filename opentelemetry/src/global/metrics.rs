use crate::metrics::{self, Meter, MeterProvider};
use crate::{otel_error, otel_info, InstrumentationScope};
use std::sync::{Arc, OnceLock, RwLock};

type GlobalMeterProvider = Arc<dyn MeterProvider + Send + Sync>;

/// The global `MeterProvider` singleton.
static GLOBAL_METER_PROVIDER: OnceLock<RwLock<GlobalMeterProvider>> = OnceLock::new();

#[inline]
fn global_meter_provider() -> &'static RwLock<GlobalMeterProvider> {
    GLOBAL_METER_PROVIDER
        .get_or_init(|| RwLock::new(Arc::new(crate::metrics::noop::NoopMeterProvider::new())))
}

/// Sets the given [`MeterProvider`] instance as the current global meter
/// provider.
pub fn set_meter_provider<P>(new_provider: P)
where
    P: metrics::MeterProvider + Send + Sync + 'static,
{
    // Try to set the global meter provider. If the RwLock is poisoned, we'll log an error.
    let mut global_provider = global_meter_provider().write();

    if let Ok(ref mut provider) = global_provider {
        **provider = Arc::new(new_provider);
        otel_info!(name: "MeterProvider.GlobalSet", message = "Global meter provider is set. Meters can now be created using global::meter() or global::meter_with_scope().");
    } else {
        otel_error!(name: "MeterProvider.GlobalSetFailed", message = "Global meter provider is not set due to lock poison. Meters created using global::meter() or global::meter_with_scope() will not function.");
    }
}

/// Returns an instance of the currently configured global [`MeterProvider`].
pub fn meter_provider() -> GlobalMeterProvider {
    global_meter_provider()
        .read()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned")
        .clone()
}

/// Creates a named [`Meter`] via the currently configured global [`MeterProvider`].
///
/// If the name is an empty string, the provider will use a default name.
///
/// This is a more convenient way of expressing `global::meter_provider().meter(name)`.
pub fn meter(name: &'static str) -> Meter {
    meter_provider().meter(name)
}

/// Creates a [`Meter`] with the given instrumentation scope.
///
/// This is a simpler alternative to `global::meter_provider().meter_with_scope(...)`
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use opentelemetry::global::meter_with_scope;
/// use opentelemetry::InstrumentationScope;
/// use opentelemetry::KeyValue;
///
/// let scope = InstrumentationScope::builder("io.opentelemetry")
///     .with_version("0.17")
///     .with_schema_url("https://opentelemetry.io/schema/1.2.0")
///     .with_attributes(vec![(KeyValue::new("key", "value"))])
///     .build();
///
/// let meter = meter_with_scope(scope);
/// ```
pub fn meter_with_scope(scope: InstrumentationScope) -> Meter {
    meter_provider().meter_with_scope(scope)
}
