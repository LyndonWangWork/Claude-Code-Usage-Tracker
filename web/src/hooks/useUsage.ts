/**
 * Custom hooks for accessing Claude Code usage data via Tauri IPC
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useState, useEffect, useCallback, useRef } from 'react';

// Types matching the Rust backend
export interface ProjectStats {
  projectPath: string;
  displayName: string;
  totalInputTokens: number;
  totalOutputTokens: number;
  cacheCreationTokens: number;
  cacheReadTokens: number;
  totalCostUsd: number;
  messageCount: number;
  sessionCount: number;
  firstActivity: string | null;
  lastActivity: string | null;
}

export interface DailyUsage {
  date: string;
  inputTokens: number;
  outputTokens: number;
  cacheCreationTokens: number;
  cacheReadTokens: number;
  costUsd: number;
  messageCount: number;
}

export interface ModelStats {
  model: string;
  inputTokens: number;
  outputTokens: number;
  cacheCreationTokens: number;
  cacheReadTokens: number;
  totalTokens: number;
  costUsd: number;
  messageCount: number;
  percentage: number;
}

export interface BurnRate {
  tokensPerMinute: number;
  costPerHour: number;
}

export interface TodayStats {
  costUsd: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  messageCount: number;
}

export interface OverallStats {
  totalInputTokens: number;
  totalOutputTokens: number;
  cacheCreationTokens: number;
  cacheReadTokens: number;
  totalCostUsd: number;
  totalMessages: number;
  totalSessions: number;
  projectCount: number;
  // Advanced metrics
  modelDistribution: ModelStats[];
  sessionStartTime: string | null;
  timeToResetMinutes: number;
  burnRate: BurnRate | null;
  todayStats: TodayStats;
}

export interface UsageData {
  projects: ProjectStats[];
  dailyUsage: DailyUsage[];
  overallStats: OverallStats;
}

/** Incremental update payload from backend push notifications */
export interface UsageDataDelta {
  /** Whether there are actual data changes (triggers animation) */
  hasChanges: boolean;
  /** Whether frontend should do a full refresh */
  fullRefresh: boolean;
  /** Projects that have been updated */
  updatedProjects: ProjectStats[];
  /** Updated overall statistics (if changed) */
  overallStats: OverallStats | null;
  /** Updated daily usage (if changed) */
  dailyUsage: DailyUsage[] | null;
}

/** Event name for usage data updates (must match backend) */
const USAGE_DATA_UPDATED_EVENT = 'usage-data-updated';

export interface AppConfig {
  dataPath: string | null;
  refreshIntervalSeconds: number;
  planType: string;
}

interface UseAsyncState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

/**
 * Hook to fetch complete usage statistics
 */
export function useUsageStats(dataPath?: string): UseAsyncState<UsageData> {
  const [data, setData] = useState<UsageData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<UsageData>('get_usage_stats', {
        dataPath: dataPath || null,
      });
      setData(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [dataPath]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook to fetch projects list
 */
export function useProjects(dataPath?: string): UseAsyncState<ProjectStats[]> {
  const [data, setData] = useState<ProjectStats[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ProjectStats[]>('get_projects', {
        dataPath: dataPath || null,
      });
      setData(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [dataPath]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook to fetch overall statistics
 */
export function useOverallStats(dataPath?: string): UseAsyncState<OverallStats> {
  const [data, setData] = useState<OverallStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<OverallStats>('get_overall_stats', {
        dataPath: dataPath || null,
      });
      setData(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [dataPath]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook to fetch daily usage data
 */
export function useDailyUsage(
  dataPath?: string,
  startDate?: string,
  endDate?: string
): UseAsyncState<DailyUsage[]> {
  const [data, setData] = useState<DailyUsage[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DailyUsage[]>('get_daily_usage', {
        dataPath: dataPath || null,
        startDate: startDate || null,
        endDate: endDate || null,
      });
      setData(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [dataPath, startDate, endDate]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook to check if the data directory exists
 */
export function useDataDirectoryCheck(dataPath?: string): UseAsyncState<boolean> {
  const [data, setData] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<boolean>('check_data_directory', {
        dataPath: dataPath || null,
      });
      setData(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [dataPath]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook for auto-refreshing usage data
 */
export function useAutoRefreshUsage(
  intervalMs: number = 300000, // 5 minutes default
  dataPath?: string
): UseAsyncState<UsageData> & { isAutoRefreshing: boolean; toggleAutoRefresh: () => void } {
  const [isAutoRefreshing, setIsAutoRefreshing] = useState(true);
  const { data, loading, error, refetch } = useUsageStats(dataPath);

  useEffect(() => {
    if (!isAutoRefreshing) return;

    const intervalId = setInterval(() => {
      refetch();
    }, intervalMs);

    return () => clearInterval(intervalId);
  }, [isAutoRefreshing, intervalMs, refetch]);

  const toggleAutoRefresh = useCallback(() => {
    setIsAutoRefreshing((prev) => !prev);
  }, []);

  return { data, loading, error, refetch, isAutoRefreshing, toggleAutoRefresh };
}

/**
 * Hook for incremental usage data refresh
 * - First load: full refresh
 * - Auto refresh every 5 seconds: incremental refresh (silent, no loading state)
 * - Manual refresh button: full refresh (shows loading state)
 */
export function useIncrementalUsageStats(
  dataPath?: string,
  autoRefreshIntervalMs: number = 5000 // 5 seconds default
): UseAsyncState<UsageData> & {
  fullRefetch: () => Promise<void>;
  isAutoRefreshing: boolean;
  refreshKey: number;
  isAnimating: boolean;
} {
  const [data, setData] = useState<UsageData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isAutoRefreshing, setIsAutoRefreshing] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const [isAnimating, setIsAnimating] = useState(false);
  const animationTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isFirstFetch = useRef(true);

  // Trigger animation for 800ms
  const triggerAnimation = useCallback(() => {
    // Clear any existing timer
    if (animationTimerRef.current) {
      clearTimeout(animationTimerRef.current);
    }
    setIsAnimating(true);
    animationTimerRef.current = setTimeout(() => {
      setIsAnimating(false);
    }, 800);
  }, []);

  // Incremental refresh (silent, no loading state change)
  const incrementalRefresh = useCallback(async () => {
    try {
      setIsAutoRefreshing(true);
      const result = await invoke<UsageData>('get_usage_stats_incremental', {
        dataPath: dataPath || null,
        forceFull: false,
      });
      setData(result);
      setError(null);
      setRefreshKey(prev => prev + 1);
      // Trigger animation on refresh (not on first fetch)
      if (!isFirstFetch.current) {
        triggerAnimation();
      }
    } catch (e) {
      // Silent error for auto-refresh, don't update error state
      console.error('Auto refresh error:', e);
    } finally {
      setIsAutoRefreshing(false);
    }
  }, [dataPath, triggerAnimation]);

  // Full refresh (shows loading state, clears cache)
  const fullRefetch = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<UsageData>('get_usage_stats_incremental', {
        dataPath: dataPath || null,
        forceFull: true,
      });
      setData(result);
      setRefreshKey(prev => prev + 1);
      // Mark first fetch as done, trigger animation on subsequent refreshes
      if (isFirstFetch.current) {
        isFirstFetch.current = false;
      } else {
        triggerAnimation();
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [dataPath, triggerAnimation]);

  // Initial load
  useEffect(() => {
    fullRefetch();
  }, [fullRefetch]);

  // Auto refresh timer (5 seconds)
  useEffect(() => {
    if (loading) return; // Don't start timer until initial load completes

    const intervalId = setInterval(() => {
      incrementalRefresh();
    }, autoRefreshIntervalMs);

    return () => clearInterval(intervalId);
  }, [loading, autoRefreshIntervalMs, incrementalRefresh]);

  return {
    data,
    loading,
    error,
    refetch: incrementalRefresh, // Default refetch is incremental
    fullRefetch, // Manual full refresh
    isAutoRefreshing,
    refreshKey, // Key that changes on each successful refresh
    isAnimating, // True for 800ms after each refresh (except first)
  };
}

/**
 * Hook for push-based usage data updates
 * - Initial load: full fetch from backend
 * - Updates: listen to backend push events (no polling)
 * - Manual refresh: full fetch with loading state
 * - Heartbeat: backend sends event every 5s even if no changes
 */
export function usePushBasedUsageStats(
  dataPath?: string
): UseAsyncState<UsageData> & {
  fullRefetch: () => Promise<void>;
  isAnimating: boolean;
  lastHeartbeat: number | null;
} {
  const [data, setData] = useState<UsageData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isAnimating, setIsAnimating] = useState(false);
  const [lastHeartbeat, setLastHeartbeat] = useState<number | null>(null);
  const animationTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isFirstFetch = useRef(true);

  // Trigger animation for 800ms
  const triggerAnimation = useCallback(() => {
    if (animationTimerRef.current) {
      clearTimeout(animationTimerRef.current);
    }
    setIsAnimating(true);
    animationTimerRef.current = setTimeout(() => {
      setIsAnimating(false);
    }, 800);
  }, []);

  // Merge delta into existing data
  const mergeDelta = useCallback((currentData: UsageData, delta: UsageDataDelta): UsageData => {
    // If full refresh requested, we should refetch
    if (delta.fullRefresh) {
      return currentData; // Will trigger full refetch
    }

    // Merge updated projects
    const projectMap = new Map(currentData.projects.map(p => [p.projectPath, p]));
    for (const updatedProject of delta.updatedProjects) {
      projectMap.set(updatedProject.projectPath, updatedProject);
    }
    const mergedProjects = Array.from(projectMap.values());

    return {
      projects: mergedProjects,
      dailyUsage: delta.dailyUsage ?? currentData.dailyUsage,
      overallStats: delta.overallStats ?? currentData.overallStats,
    };
  }, []);

  // Full refresh (shows loading state)
  const fullRefetch = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<UsageData>('get_usage_stats_incremental', {
        dataPath: dataPath || null,
        forceFull: true,
      });
      setData(result);
      if (isFirstFetch.current) {
        isFirstFetch.current = false;
      } else {
        triggerAnimation();
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [dataPath, triggerAnimation]);

  // Initial load
  useEffect(() => {
    fullRefetch();
  }, [fullRefetch]);

  // Listen for backend push events
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    const setupListener = async () => {
      unlisten = await listen<UsageDataDelta>(USAGE_DATA_UPDATED_EVENT, (event) => {
        const delta = event.payload;
        console.log('Received usage-data-updated event:', delta);

        // Update heartbeat timestamp
        setLastHeartbeat(Date.now());

        // If full refresh requested, do a full refetch
        if (delta.fullRefresh) {
          fullRefetch();
          return;
        }

        // Only process and animate if there are actual changes
        if (delta.hasChanges) {
          // Merge delta into existing data
          setData(currentData => {
            if (!currentData) return currentData;
            const merged = mergeDelta(currentData, delta);
            return merged;
          });

          // Trigger animation only when data changed
          triggerAnimation();
        }
      });
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [fullRefetch, mergeDelta, triggerAnimation]);

  // Cleanup animation timer on unmount
  useEffect(() => {
    return () => {
      if (animationTimerRef.current) {
        clearTimeout(animationTimerRef.current);
      }
    };
  }, []);

  return {
    data,
    loading,
    error,
    refetch: fullRefetch, // Manual refresh is always full
    fullRefetch,
    isAnimating,
    lastHeartbeat, // Timestamp of last heartbeat from backend
  };
}

/**
 * Format token count for display
 */
export function formatTokens(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}k`;
  }
  return count.toLocaleString();
}

/**
 * Format cost for display
 */
export function formatCost(cost: number): string {
  return `${cost.toFixed(2)}`;
}

/**
 * Format relative time from ISO timestamp
 */
export function formatRelativeTime(isoTimestamp: string | null): string {
  if (!isoTimestamp) return 'Never';

  const date = new Date(isoTimestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins} min ago`;
  if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
  if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;

  return date.toLocaleDateString();
}
