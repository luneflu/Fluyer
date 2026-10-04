using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class FloatClampTests
{
    [Fact]
    public void ConfinesToRange()
    {
        Assert.Equal(0.5f, 0.5f.Clamped(0.0f, 1.0f));
        Assert.Equal(0.0f, (-2.0f).Clamped(0.0f, 1.0f));
        Assert.Equal(1.0f, 2.0f.Clamped(0.0f, 1.0f));
    }

    [Fact]
    public void NaN_CollapsesToLowerBound()
    {
        Assert.Equal(0.0f, float.NaN.Clamped(0.0f, 1.0f));
        Assert.Equal(0.0, double.NaN.Clamped(0.0, 1.0));
    }

    [Fact]
    public void Infinities_PinToEnds()
    {
        Assert.Equal(1.0f, float.PositiveInfinity.Clamped(0.0f, 1.0f));
        Assert.Equal(0.0f, float.NegativeInfinity.Clamped(0.0f, 1.0f));
    }
}
