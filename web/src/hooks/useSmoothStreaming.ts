import { useCallback, useEffect, useRef, useState } from "react";

interface SmoothStreamingOptions {
  /** Target characters per second for steady streaming */
  baseCharsPerSecond?: number;
  /** Minimum delay between updates (ms) */
  minUpdateDelay?: number;
  /** Maximum delay between updates (ms) */
  maxUpdateDelay?: number;
  /** Buffer size threshold for speeding up */
  bufferSpeedUpThreshold?: number;
  /** Speed multiplier when buffer is full */
  speedUpMultiplier?: number;
}

interface SmoothStreamingState {
  /** The currently displayed text */
  displayedText: string;
  /** Whether streaming is in progress */
  isStreaming: boolean;
  /** Current buffer size */
  bufferSize: number;
  /** Current streaming rate (chars/sec) */
  currentRate: number;
}

/**
 * Smoothly output an incoming text stream.
 */
export function useSmoothStreaming(
  streamText?: string,
  options: SmoothStreamingOptions = {},
) {
  const {
    baseCharsPerSecond = 50,
    minUpdateDelay = 8,
    maxUpdateDelay = 80,
    bufferSpeedUpThreshold = 80,
    speedUpMultiplier = 3.0,
  } = options;

  // State
  const [displayedText, setDisplayedText] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [currentRate, setCurrentRate] = useState(baseCharsPerSecond);

  // Internal state refs
  const bufferRef = useRef<string>("");
  const positionRef = useRef<number>(0);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastUpdateTimeRef = useRef<number>(Date.now());

  // Track stream changes for rate calculation
  const lastStreamLengthRef = useRef<number>(0);
  const lastStreamTimeRef = useRef<number>(Date.now());
  const recentRatesRef = useRef<number[]>([]);

  /**
   * Calculate incoming rate based on recent chunk timing
   */
  const calculateIncomingRate = useCallback((): number => {
    const rates = recentRatesRef.current;
    if (rates.length === 0) return baseCharsPerSecond;

    // Weighted average with more recent rates getting higher weight
    let weightedSum = 0;
    let totalWeight = 0;
    for (let i = 0; i < rates.length; i++) {
      const weight = i + 1;
      weightedSum += rates[i] * weight;
      totalWeight += weight;
    }
    return weightedSum / totalWeight;
  }, [baseCharsPerSecond]);

  /**
   * Calculate optimal processing parameters based on buffer state and incoming rate
   */
  const calculateProcessingParams = useCallback(
    (bufferSize: number) => {
      const incomingRate = calculateIncomingRate();
      let targetRate = Math.max(incomingRate, baseCharsPerSecond);

      // Speed up reasonably when buffer grows
      if (bufferSize > bufferSpeedUpThreshold) {
        const speedFactor = Math.min(speedUpMultiplier, 1 + bufferSize / 20);
        targetRate = targetRate * speedFactor;
      }

      // Cap at reasonable streaming speeds
      targetRate = Math.min(targetRate, 400);
      setCurrentRate(targetRate);

      // Try to keep up with stream while still having the 'smooth' effect
      const charsPerUpdate =
        bufferSize < 20 ? 2 : bufferSize < 50 ? 3 : Math.floor(bufferSize / 3);

      const idealInterval = (1000 / targetRate) * charsPerUpdate;
      const interval = Math.max(
        minUpdateDelay,
        Math.min(maxUpdateDelay, idealInterval),
      );

      return { charsPerUpdate, interval, targetRate };
    },
    [
      calculateIncomingRate,
      baseCharsPerSecond,
      bufferSpeedUpThreshold,
      speedUpMultiplier,
      minUpdateDelay,
      maxUpdateDelay,
    ],
  );

  /**
   * Process characters from buffer
   */
  const processChars = useCallback(() => {
    const buffer = bufferRef.current;
    const position = positionRef.current;

    if (position >= buffer.length) {
      // Caught up to buffer - pause for more chunks
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
      return;
    }

    const remainingBufferSize = buffer.length - position;
    const { charsPerUpdate, interval } =
      calculateProcessingParams(remainingBufferSize);

    // Process multiple characters for smoother catching up
    const actualCharsToProcess = Math.min(charsPerUpdate, remainingBufferSize);
    positionRef.current += actualCharsToProcess;

    // Update displayed text
    setDisplayedText(buffer.substring(0, positionRef.current));

    // Schedule next processing
    timeoutRef.current = setTimeout(processChars, interval);
    lastUpdateTimeRef.current = Date.now();
  }, [calculateProcessingParams]);

  /**
   * Detect stream changes and update buffer
   */
  useEffect(() => {
    const now = Date.now();
    const newLength = streamText?.length ?? 0;
    const prevLength = lastStreamLengthRef.current;

    // Only process if stream has grown
    if (newLength > prevLength) {
      const newChars = newLength - prevLength;
      const timeDiff = now - lastStreamTimeRef.current;

      // Calculate rate if we have timing data
      if (timeDiff > 0 && prevLength > 0) {
        const rate = (newChars / timeDiff) * 1000;
        recentRatesRef.current.push(rate);

        // Keep only recent rates
        if (recentRatesRef.current.length > 5) {
          recentRatesRef.current.shift();
        }
      }

      // Update buffer
      bufferRef.current = streamText || "";
      lastStreamLengthRef.current = newLength;
      lastStreamTimeRef.current = now;

      // If previous stream length was empty and newLength is big,
      // we might be switching to an ongoing stream. Skip ahead to the
      // current position instead of animating all of the characters so far.
      if (prevLength === 0 && newLength > 100) {
        positionRef.current = bufferRef.current.length - 2;
      }

      // Start or resume streaming
      if (!isStreaming && timeoutRef.current === null) {
        setIsStreaming(true);
        processChars();
      } else if (isStreaming && timeoutRef.current === null) {
        // Resume if paused
        processChars();
      }
    }
  }, [streamText, processChars, isStreaming]);

  /**
   * Reset the streaming state
   */
  const reset = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }

    bufferRef.current = "";
    positionRef.current = 0;
    setDisplayedText("");
    setIsStreaming(false);
    setCurrentRate(baseCharsPerSecond);
    lastUpdateTimeRef.current = Date.now();

    // Reset timing tracking
    lastStreamLengthRef.current = 0;
    lastStreamTimeRef.current = Date.now();
    recentRatesRef.current = [];
  }, [baseCharsPerSecond]);

  /**
   * Complete streaming immediately (show all buffered text)
   */
  const complete = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }

    const buffer = bufferRef.current;
    setDisplayedText(buffer);
    positionRef.current = buffer.length;
    setIsStreaming(false);
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  // Calculate current buffer size for external monitoring
  const bufferSize = bufferRef.current.length - positionRef.current;

  const state: SmoothStreamingState = {
    displayedText,
    isStreaming,
    bufferSize,
    currentRate,
  };

  return {
    ...state,
    reset,
    complete,
  };
}

export default useSmoothStreaming;
