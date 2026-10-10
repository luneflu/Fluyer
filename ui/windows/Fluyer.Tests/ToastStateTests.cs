using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class ToastStateTests
{
    [Fact]
    public void Show_SetsMessage()
    {
        var toast = new ToastState();
        toast.Show("hello");
        Assert.Equal("hello", toast.Message);
        toast.Dismiss();
    }

    [Fact]
    public void Replace_Supersedes()
    {
        var toast = new ToastState();
        toast.Show("first");
        toast.Show("second");
        Assert.Equal("second", toast.Message);
        toast.Dismiss();
    }

    [Fact]
    public void Dismiss_Clears()
    {
        var toast = new ToastState();
        toast.Show("hello");
        toast.Dismiss();
        Assert.Null(toast.Message);
    }

    [Fact]
    public async Task RapidToasts_NoPileup()
    {
        var toast = new ToastState();
        for (var i = 0; i < 50; i++)
        {
            toast.Show($"m{i}");
        }
        Assert.Equal("m49", toast.Message);
        toast.Dismiss();
        await Task.Delay(50);
        Assert.Null(toast.Message);
    }
}
