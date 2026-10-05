using Fluyer.Core.State;

namespace Fluyer.Tests;

public sealed class QueueStateTests
{
    private static (QueueState State, FakeEngine Engine) Create(int count)
    {
        var engine = new FakeEngine();
        for (var i = 0; i < count; i++)
        {
            engine.QueueList.Add(FakeEngine.Track((ulong)i));
        }
        return (new QueueState { Engine = engine }, engine);
    }

    [Fact]
    public void Closed_DoesNotReadQueue_OpenLoadsIt()
    {
        var (state, _) = Create(3);
        state.Reload();
        Assert.Empty(state.Tracks);
        state.IsOpen = true;
        Assert.Equal(new[] { "T0", "T1", "T2" }, state.Tracks.Select(t => t.Title));
    }

    [Fact]
    public void Move_ClampsAtEnds_AndRefreshesOrder()
    {
        var (state, engine) = Create(3);
        state.IsOpen = true;

        state.Move(0, -1);
        state.Move(2, 1);
        Assert.Empty(engine.Calls);

        state.Move(0, 1);
        Assert.Equal(new[] { "move:0:1" }, engine.Calls);
        Assert.Equal(new[] { "T1", "T0", "T2" }, state.Tracks.Select(t => t.Title));
    }

    [Fact]
    public void Remove_And_Goto_IgnoreOutOfRange()
    {
        var (state, engine) = Create(2);
        state.IsOpen = true;

        state.Remove(5);
        state.Goto(-1);
        state.Goto(2);
        Assert.Empty(engine.Calls);

        state.Remove(0);
        state.Goto(0);
        Assert.Equal(new[] { "remove:0", "goto:0" }, engine.Calls);
        Assert.Equal(new[] { "T1" }, state.Tracks.Select(t => t.Title));
    }
}
