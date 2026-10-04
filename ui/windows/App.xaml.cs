using Fluyer.Core.State;
using Fluyer.Support;
using Microsoft.UI.Xaml;

namespace Fluyer;

/// <summary>
/// App entry point. Creates the shared <see cref="ThumbnailStore"/>, builds the
/// root <see cref="AppState"/>, then starts the core against it (the state is
/// the core's event sink, so it must exist first — mirrors the macOS
/// <c>EngineHandle.attach(listener:)</c> ordering).
/// </summary>
public partial class App : Application
{
    public AppState State { get; }
    public ThumbnailStore Thumbnails { get; } = ThumbnailStore.Shared;

    private Window? _window;

    public App()
    {
        InitializeComponent();

        // The engine hops callbacks onto the constructing thread's
        // SynchronizationContext — make sure the UI thread has one before the
        // core starts emitting.
        if (SynchronizationContext.Current is null)
        {
            var queue = Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread();
            if (queue is not null)
            {
                SynchronizationContext.SetSynchronizationContext(
                    new Microsoft.UI.Dispatching.DispatcherQueueSynchronizationContext(queue));
            }
        }

        State = new AppState(null, Thumbnails);
        State.AttachEngine(EngineHandle.Attach(State));
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow(State);
        _window.Activate();
    }
}
