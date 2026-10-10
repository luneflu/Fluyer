using Fluyer.Core.State;
using Fluyer.Shared;
using Microsoft.UI.Xaml;

namespace Fluyer;

/// <summary>
/// App entry point. Builds the root <see cref="AppState"/> and the
/// <see cref="CoverState"/> (its thumbnail invalidator), then starts the core
/// against the state (the state is the core's event sink, so it must exist
/// first — mirrors the macOS <c>EngineHandle.attach(listener:)</c> ordering).
/// </summary>
public partial class App : Application
{
    public AppState State { get; }
    public CoverState Covers { get; } = new();

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

        var settings = new SettingsState(Path.Combine(EngineHandle.DataDirectory, SettingsState.FileName));
        State = new AppState(null, Covers, settings);
        var engine = EngineHandle.Attach(State);
        Covers.Engine = engine;
        State.AttachEngine(engine);
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow(State, Covers);
        _window.Closed += (_, _) => State.SaveSession();
        _window.Activate();
    }
}
