namespace Fluyer.Core.State;

/// <summary>
/// Transient status messages from the core. Port of <c>ToastState</c>
/// (<c>ui/macos/Sources/Shared/State/ToastState.swift</c>): replacing a toast cancels
/// the pending dismissal so a superseded message cannot clear its successor.
/// </summary>
public sealed class ToastState : Support.ObservableObject
{
    private static readonly TimeSpan DisplayDuration = TimeSpan.FromSeconds(3);

    private string? _message;
    private CancellationTokenSource? _dismissal;
    private SynchronizationContext? _context;

    public string? Message
    {
        get => _message;
        private set => SetProperty(ref _message, value);
    }

    public void Show(string message)
    {
        // Called on the UI thread in the app; capture it so the delayed clear
        // also lands on the UI thread (WinUI bindings require it).
        _context = SynchronizationContext.Current;
        Message = message;
        _dismissal?.Cancel();
        _dismissal?.Dispose();
        var cts = new CancellationTokenSource();
        _dismissal = cts;
        var context = _context;
        Task.Delay(DisplayDuration, cts.Token).ContinueWith(t =>
        {
            if (t.IsCanceled)
            {
                return;
            }
            if (context is null)
            {
                Message = null;
            }
            else
            {
                context.Post(_ => Message = null, null);
            }
        }, TaskScheduler.Default);
    }

    public void Dismiss()
    {
        _dismissal?.Cancel();
        _dismissal?.Dispose();
        _dismissal = null;
        Message = null;
    }
}
