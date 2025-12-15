import {
  DollarSign,
  Zap,
  MessageSquare,
  TrendingUp,
  Activity,
  AlertCircle,
  Loader2,
  RefreshCw,
  Clock,
  Flame,
  PieChart as PieChartIcon,
  Calendar,
} from "lucide-react";
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

function formatTimeToReset(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins}m`;
}

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
  const timeProgressPercent = ((300 - overallStats.timeToResetMinutes) / 300) * 100;

  const UsageCard = ({
    icon: Icon,
    label,
    numericValue,
    formatter,
    subValue,
    iconColor,
  }: {
    icon: any;
    label: string;
    numericValue: number;
    formatter: (value: number) => string;
    subValue?: string;
    iconColor: string;
  }) => (
    <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
      <div className="flex items-center gap-3 mb-2">
        <Icon className={`w-5 h-5 ${iconColor}`} />
        <span className="text-gray-400">{label}</span>
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

      {/* Today's Stats Row */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <UsageCard
          icon={Calendar}
          label="Today's Cost"
          numericValue={overallStats.todayStats?.costUsd || 0}
          formatter={formatCost}
          subValue={`${overallStats.todayStats?.messageCount || 0} messages today`}
          iconColor="text-cyan-400"
        />
        <UsageCard
          icon={Calendar}
          label="Today's Tokens"
          numericValue={overallStats.todayStats?.totalTokens || 0}
          formatter={formatTokens}
          subValue={`In: ${formatTokens(overallStats.todayStats?.inputTokens || 0)} / Out: ${formatTokens(overallStats.todayStats?.outputTokens || 0)}`}
          iconColor="text-cyan-400"
        />
      </div>

      {/* Session Metrics Row */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Time to Reset */}
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

        {/* Burn Rate */}
        <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
          <div className="flex items-center gap-3 mb-4">
            <Flame className="w-5 h-5 text-red-400" />
            <span className="text-gray-400">Burn Rate</span>
          </div>
          <p className="text-3xl text-gray-100">
            {overallStats.burnRate ? (
              <AnimatedNumber
                value={overallStats.burnRate.tokensPerMinute}
                formatter={(v) => `${formatTokens(v)}/min`}
              />
            ) : (
              <span className="text-gray-500">--</span>
            )}
          </p>
          <p className="text-sm text-gray-500 mt-1">
            {overallStats.burnRate
              ? `Current session consumption rate`
              : "No active session"}
          </p>
        </div>

        {/* Cost Rate */}
        <div className="border border-gray-700 rounded-lg p-6 bg-gray-900/50">
          <div className="flex items-center gap-3 mb-4">
            <DollarSign className="w-5 h-5 text-emerald-400" />
            <span className="text-gray-400">Cost Rate</span>
          </div>
          <p className="text-3xl text-gray-100">
            {overallStats.burnRate ? (
              overallStats.burnRate.costPerHour < 0.01 ? (
                <span>{"< $0.01/hr"}</span>
              ) : (
                <AnimatedNumber
                  value={overallStats.burnRate.costPerHour}
                  formatter={(v) => `$${v.toFixed(2)}/hr`}
                />
              )
            ) : (
              <span className="text-gray-500">--</span>
            )}
          </p>
          <p className="text-sm text-gray-500 mt-1">
            {overallStats.burnRate
              ? "Projected hourly cost"
              : "No active session"}
          </p>
        </div>
      </div>

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
