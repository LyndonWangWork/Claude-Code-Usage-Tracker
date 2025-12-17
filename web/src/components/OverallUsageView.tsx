import {
  DollarSign,
  Zap,
  MessageSquare,
  TrendingUp,
  Activity,
  AlertCircle,
  Loader2,
  RefreshCw,
  // Clock, // Hidden temporarily - telemetry doesn't support time to reset
  Flame,
  PieChart as PieChartIcon,
  Calendar,
} from "lucide-react";

// OpenTelemetry icon for telemetry data indicator
const TelemetryBadge = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 128 128"
    className="w-4 h-4"
    title="Data from telemetry"
  >
    <path
      fill="#f5a800"
      d="M67.648 69.797c-5.246 5.25-5.246 13.758 0 19.008c5.25 5.246 13.758 5.246 19.004 0c5.25-5.25 5.25-13.758 0-19.008c-5.246-5.246-13.754-5.246-19.004 0m14.207 14.219a6.65 6.65 0 0 1-9.41 0a6.65 6.65 0 0 1 0-9.407a6.65 6.65 0 0 1 9.41 0c2.598 2.586 2.598 6.809 0 9.407M86.43 3.672l-8.235 8.234a4.17 4.17 0 0 0 0 5.875l32.149 32.149a4.17 4.17 0 0 0 5.875 0l8.234-8.235c1.61-1.61 1.61-4.261 0-5.87L92.29 3.671a4.16 4.16 0 0 0-5.86 0ZM28.738 108.895a3.763 3.763 0 0 0 0-5.31l-4.183-4.187a3.77 3.77 0 0 0-5.313 0l-8.644 8.649l-.016.012l-2.371-2.375c-1.313-1.313-3.45-1.313-4.75 0c-1.313 1.312-1.313 3.449 0 4.75l14.246 14.242a3.353 3.353 0 0 0 4.746 0c1.3-1.313 1.313-3.45 0-4.746l-2.375-2.375l.016-.012Zm0 0"
    />
    <path
      fill="#425cc7"
      d="M72.297 27.313L54.004 45.605c-1.625 1.625-1.625 4.301 0 5.926L65.3 62.824c7.984-5.746 19.18-5.035 26.363 2.153l9.148-9.149c1.622-1.625 1.622-4.297 0-5.922L78.22 27.313a4.185 4.185 0 0 0-5.922 0ZM60.55 67.585l-6.672-6.672c-1.563-1.562-4.125-1.562-5.684 0l-23.53 23.54a4.036 4.036 0 0 0 0 5.687l13.331 13.332a4.036 4.036 0 0 0 5.688 0l15.132-15.157c-3.199-6.609-2.625-14.593 1.735-20.73m0 0"
    />
  </svg>
);
import React from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import {
  UsageData,
  formatTokens,
  formatCost,
  ModelStats,
} from "../hooks/useUsage";
import { AnimatedNumber } from "./AnimatedNumber";

interface OverallUsageViewProps {
  data: UsageData | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  isAnimating?: boolean;
}

// Colors for pie chart
const MODEL_COLORS: Record<string, string> = {
  "claude-3-opus": "#8B5CF6",
  "claude-opus-4-": "#A78BFA",
  "claude-3-5-sonnet": "#F59E0B",
  "claude-3-sonnet": "#FBBF24",
  "claude-sonnet-4-": "#FCD34D",
  "claude-3-5-haiku": "#10B981",
  "claude-3-haiku": "#34D399",
  "claude-haiku-4-": "#6EE7B7",
};

const DEFAULT_COLORS = ["#6366F1", "#EC4899", "#14B8A6", "#F97316", "#8B5CF6", "#06B6D4"];

function getModelColor(model: string, index: number): string {
  const modelLower = model.toLowerCase();
  for (const [key, color] of Object.entries(MODEL_COLORS)) {
    if (modelLower.includes(key)) {
      return color;
    }
  }
  return DEFAULT_COLORS[index % DEFAULT_COLORS.length];
}

function formatModelName(model: string): string {
  // Make model names more readable
  return model
    .replace("claude-", "")
    .replace("-", " ")
    .split(" ")
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

// Hidden temporarily - telemetry doesn't support time to reset
// function formatTimeToReset(minutes: number): string {
//   const hours = Math.floor(minutes / 60);
//   const mins = minutes % 60;
//   if (hours > 0) {
//     return `${hours}h ${mins}m`;
//   }
//   return `${mins}m`;
// }

export function OverallUsageView({ data, loading, error, refetch }: OverallUsageViewProps) {

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="w-8 h-8 text-blue-400 animate-spin" />
        <span className="ml-3 text-gray-400">Loading usage data...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="border border-red-700 rounded-lg p-6 bg-red-900/20">
        <div className="flex items-center gap-3 mb-4">
          <AlertCircle className="w-6 h-6 text-red-400" />
          <h3 className="text-red-400">Error loading usage data</h3>
        </div>
        <p className="text-gray-400 mb-4">{error}</p>
        <p className="text-gray-500 text-sm mb-4">
          Make sure Claude Code is installed and you have used it at least once.
          The data directory should be at ~/.claude/projects/
        </p>
        <button
          onClick={refetch}
          className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-300"
        >
          <RefreshCw className="w-4 h-4" />
          Retry
        </button>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50 text-center">
        <p className="text-gray-400">No usage data available</p>
      </div>
    );
  }

  const { overallStats, dailyUsage } = data;
  const totalTokens = overallStats.totalInputTokens + overallStats.totalOutputTokens;

  // Prepare chart data (last 30 days)
  const chartData = dailyUsage.slice(-30).map((day) => ({
    date: day.date.slice(5), // MM-DD format
    tokens: day.inputTokens + day.outputTokens,
    cost: day.costUsd,
  }));

  // Prepare pie chart data
  const pieData = overallStats.modelDistribution?.map((m: ModelStats, index: number) => ({
    name: formatModelName(m.model),
    value: m.totalTokens,
    percentage: m.percentage,
    cost: m.costUsd,
    color: getModelColor(m.model, index),
  })) || [];

  // Calculate time progress percentage (out of 5 hours = 300 minutes)
  // Hidden temporarily - telemetry doesn't support time to reset
  // const timeProgressPercent = ((300 - overallStats.timeToResetMinutes) / 300) * 100;

  // Check if data source is telemetry
  const isTelemetrySource = data?.dataSource?.sourceType === "telemetry";

  const UsageCard = ({
    icon: Icon,
    label,
    numericValue,
    formatter,
    subValue,
    iconColor,
    showTelemetryBadge = false,
  }: {
    icon: any;
    label: string;
    numericValue: number;
    formatter: (value: number) => string;
    subValue?: string;
    iconColor: string;
    showTelemetryBadge?: boolean;
  }) => (
    <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
      <div className="flex items-center gap-3 mb-2">
        <Icon className={`w-5 h-5 ${iconColor}`} />
        <span className="text-gray-400">{label}</span>
        {showTelemetryBadge && isTelemetrySource && (
          <span className="ml-auto" title="Data from telemetry">
            <TelemetryBadge />
          </span>
        )}
      </div>
      <p className="text-3xl text-gray-100">
        <AnimatedNumber value={numericValue} formatter={formatter} />
      </p>
      {subValue && <p className="text-sm text-gray-500 mt-1">{subValue}</p>}
    </div>
  );

  return (
    <div className="space-y-8">
      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <UsageCard
          icon={DollarSign}
          label="Total Cost"
          numericValue={overallStats.totalCostUsd}
          formatter={formatCost}
          subValue={`${overallStats.projectCount} projects`}
          iconColor="text-green-400"
        />
        <UsageCard
          icon={Zap}
          label="Total Tokens"
          numericValue={totalTokens}
          formatter={formatTokens}
          subValue={`In: ${formatTokens(overallStats.totalInputTokens)} / Out: ${formatTokens(overallStats.totalOutputTokens)}`}
          iconColor="text-yellow-400"
        />
        <UsageCard
          icon={MessageSquare}
          label="Total Messages"
          numericValue={overallStats.totalMessages}
          formatter={(v) => Math.round(v).toLocaleString()}
          subValue={`${overallStats.totalSessions} sessions`}
          iconColor="text-blue-400"
        />
        <UsageCard
          icon={Activity}
          label="Cache Tokens"
          numericValue={overallStats.cacheCreationTokens + overallStats.cacheReadTokens}
          formatter={formatTokens}
          subValue={`Create: ${formatTokens(overallStats.cacheCreationTokens)} / Read: ${formatTokens(overallStats.cacheReadTokens)}`}
          iconColor="text-purple-400"
        />
      </div>

      {/* Today's Stats & Burn Rate Row - 4 columns */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <UsageCard
          icon={Calendar}
          label="Today's Cost"
          numericValue={overallStats.todayStats?.costUsd || 0}
          formatter={formatCost}
          subValue={`${overallStats.todayStats?.messageCount || 0} msgs today`}
          iconColor="text-cyan-400"
          showTelemetryBadge
        />
        <UsageCard
          icon={Calendar}
          label="Today's Tokens"
          numericValue={overallStats.todayStats?.totalTokens || 0}
          formatter={formatTokens}
          subValue={`In: ${formatTokens(overallStats.todayStats?.inputTokens || 0)} / Out: ${formatTokens(overallStats.todayStats?.outputTokens || 0)}`}
          iconColor="text-cyan-400"
          showTelemetryBadge
        />
        <UsageCard
          icon={Flame}
          label="Burn Rate"
          numericValue={overallStats.burnRate?.tokensPerMinute || 0}
          formatter={(v) => v > 0 ? `${formatTokens(v)}/min` : "--"}
          subValue={overallStats.burnRate ? "Current consumption" : "No active session"}
          iconColor="text-red-400"
          showTelemetryBadge
        />
        <UsageCard
          icon={DollarSign}
          label="Cost Rate"
          numericValue={overallStats.burnRate?.costPerHour || 0}
          formatter={(v) => v > 0 ? (v < 0.01 ? "< $0.01/hr" : `$${v.toFixed(2)}/hr`) : "--"}
          subValue={overallStats.burnRate ? "Projected hourly" : "No active session"}
          iconColor="text-emerald-400"
          showTelemetryBadge
        />
      </div>

      {/* Time to Reset - Hidden temporarily, telemetry doesn't support this */}
      {/* <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
          <div className="flex items-center gap-3 mb-4">
            <Clock className="w-5 h-5 text-orange-400" />
            <span className="text-gray-400">Time to Reset</span>
          </div>
          <p className="text-3xl text-gray-100 mb-3">
            {formatTimeToReset(overallStats.timeToResetMinutes)}
          </p>
          <div className="w-full bg-gray-700 rounded-full h-2">
            <div
              className="bg-orange-400 h-2 rounded-full transition-all duration-300"
              style={{ width: `${timeProgressPercent}%` }}
            />
          </div>
          <p className="text-sm text-gray-500 mt-2">5-hour session window</p>
        </div>
      </div> */}

      {/* Model Distribution */}
      {pieData.length > 0 && (
        <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
          <div className="flex items-center gap-2 mb-6">
            <PieChartIcon className="w-5 h-5 text-indigo-400" />
            <h3 className="text-gray-300">Model Distribution</h3>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Pie Chart */}
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={pieData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={100}
                    paddingAngle={2}
                    dataKey="value"
                  >
                    {pieData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "#1F2937",
                      border: "1px solid #374151",
                      borderRadius: "8px",
                    }}
                    itemStyle={{ color: "#E5E7EB" }}
                    labelStyle={{ color: "#9CA3AF" }}
                    formatter={(value: number, name: string, props: any) => [
                      `${formatTokens(value)} (${props.payload.percentage.toFixed(1)}%)`,
                      props.payload.name,
                    ]}
                  />
                </PieChart>
              </ResponsiveContainer>
            </div>
            {/* Legend */}
            <div className="space-y-3">
              {pieData.map((model, index) => (
                <div key={index} className="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg">
                  <div className="flex items-center gap-3">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: model.color }}
                    />
                    <span className="text-gray-300">{model.name}</span>
                  </div>
                  <div className="text-right">
                    <div className="text-gray-200">{model.percentage.toFixed(1)}%</div>
                    <div className="text-sm text-gray-500">{formatTokens(model.value)} tokens</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Usage Trend Chart */}
      {chartData.length > 0 && (
        <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
          <div className="flex items-center gap-2 mb-6">
            <TrendingUp className="w-5 h-5 text-yellow-400" />
            <h3 className="text-gray-300">Usage Trend (Last 30 Days)</h3>
          </div>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis
                  dataKey="date"
                  stroke="#9CA3AF"
                  fontSize={12}
                  tickLine={false}
                />
                <YAxis
                  stroke="#9CA3AF"
                  fontSize={12}
                  tickLine={false}
                  tickFormatter={(value) => formatTokens(value)}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1F2937",
                    border: "1px solid #374151",
                    borderRadius: "8px",
                  }}
                  labelStyle={{ color: "#9CA3AF" }}
                  formatter={(value: number, name: string) => [
                    name === "tokens"
                      ? formatTokens(value)
                      : formatCost(value),
                    name === "tokens" ? "Tokens" : "Cost",
                  ]}
                />
                <Line
                  type="monotone"
                  dataKey="tokens"
                  stroke="#EAB308"
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      )}

      {/* Token Breakdown */}
      <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
        <div className="flex items-center gap-2 mb-4">
          <Zap className="w-5 h-5 text-yellow-400" />
          <h3 className="text-gray-300">Token Breakdown</h3>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="p-4 bg-gray-800/50 rounded-lg">
            <div className="text-gray-500 text-sm mb-1">Input Tokens</div>
            <div className="text-xl text-gray-200">
              <AnimatedNumber value={overallStats.totalInputTokens} formatter={formatTokens} />
            </div>
          </div>
          <div className="p-4 bg-gray-800/50 rounded-lg">
            <div className="text-gray-500 text-sm mb-1">Output Tokens</div>
            <div className="text-xl text-gray-200">
              <AnimatedNumber value={overallStats.totalOutputTokens} formatter={formatTokens} />
            </div>
          </div>
          <div className="p-4 bg-gray-800/50 rounded-lg">
            <div className="text-gray-500 text-sm mb-1">Cache Creation</div>
            <div className="text-xl text-gray-200">
              <AnimatedNumber value={overallStats.cacheCreationTokens} formatter={formatTokens} />
            </div>
          </div>
          <div className="p-4 bg-gray-800/50 rounded-lg">
            <div className="text-gray-500 text-sm mb-1">Cache Read</div>
            <div className="text-xl text-gray-200">
              <AnimatedNumber value={overallStats.cacheReadTokens} formatter={formatTokens} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
