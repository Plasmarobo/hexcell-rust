use embedded_error_chain::ErrorCategory;

#[derive(Clone, Copy, ErrorCategory)]
#[repr(u8)]
pub enum PhyError {
  UartError,
  SPIError,
  DMAError,
  NotConnected,
  InvalidPort,
  #[error("{variant}, local port is not accepting connections (busy)")]
  LocalResourceBusy,
  #[error("{variant}, remote port is not accepting connections (busy)")]
  RemoteResourceBusy,
  InvalidSignal,
}

#[derive(Clone, Copy, ErrorCategory)]
#[error_category(links(PhyError))]
#[repr(u8)]
pub enum NetworkError {
  InvalidAddress,
  DestinationUnreachable,
  ChecksumFailure,
  Timeout,
  InvalidConfiguration,
  InvalidMessageContents
}

#[derive(Clone, Copy, ErrorCategory)]
#[repr(u8)]
pub enum GenError
{
  #[error("{variant}, disconnect a port before connect")]
  ExistingConnectionError,
  #[error("{variant}, no messages in queue")]
  EmptyQueueError,
}
