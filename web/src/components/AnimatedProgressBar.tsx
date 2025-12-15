import React from "react";

interface AnimatedProgressBarProps {
  value: number;
  maxValue: number;
  color: "green" | "yellow" | "blue";
  height?: string;
  /** Whether to show animation effect */
  isAnimating?: boolean;
}

export function AnimatedProgressBar({
  value,
  maxValue,
  color,
  height = "h-1",
  isAnimating = false,
}: AnimatedProgressBarProps) {
  const percentage = maxValue > 0 ? (value / maxValue) * 100 : 0;

  // Color mappings - use brighter colors when animating
  const colorClasses = {
    green: {
      bg: "bg-green-500",
      animateBg: "bg-green-300",
      glow: "#22c55e",
      shimmer: "rgba(134, 239, 172, 0.8)",
    },
    yellow: {
      bg: "bg-yellow-500",
      animateBg: "bg-yellow-200",
      glow: "#eab308",
      shimmer: "rgba(254, 240, 138, 0.8)",
    },
    blue: {
      bg: "bg-blue-500",
      animateBg: "bg-blue-300",
      glow: "#3b82f6",
      shimmer: "rgba(147, 197, 253, 0.8)",
    },
  };

  const colors = colorClasses[color];

  return (
    <div className={`${height} bg-gray-700 rounded-full overflow-hidden relative`}>
      <div
        className={`h-full transition-all duration-300 ${
          isAnimating ? colors.animateBg : colors.bg
        }`}
        style={{
          width: `${percentage}%`,
          boxShadow: isAnimating ? `0 0 12px 4px ${colors.glow}, 0 0 20px 6px ${colors.glow}40` : 'none',
        }}
      />
      {/* Pulse glow effect */}
      {isAnimating && (
        <div
          className="absolute inset-0 rounded-full animate-pulse"
          style={{
            width: `${percentage}%`,
            backgroundColor: colors.glow,
            opacity: 0.6,
          }}
        />
      )}
      {/* Shimmer sweep effect */}
      {isAnimating && (
        <div
          className="absolute inset-0 rounded-full overflow-hidden"
          style={{ width: `${percentage}%` }}
        >
          <div
            className="h-full w-1/2 absolute animate-shimmer"
            style={{
              background: `linear-gradient(90deg, transparent, ${colors.shimmer}, transparent)`,
            }}
          />
        </div>
      )}
    </div>
  );
}
