import { Settings } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface Props {
  currentMaxTokens: number;
  onSelectMaxTokens: (tokens: number) => void;
  currentTemperature: number;
  onSelectTemperature: (temperature: number) => void;
}

export default function ChatMoreSettings({
  currentMaxTokens,
  onSelectMaxTokens,
  currentTemperature,
  onSelectTemperature,
}: Props) {
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button aria-label="More settings" size="icon" variant="outline">
          <Settings />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="flex flex-col gap-2">
        <Label>
          Max tokens
          <Select
            value={currentMaxTokens.toString()}
            onValueChange={(tokens) => onSelectMaxTokens(+tokens)}
          >
            <SelectTrigger className="w-[100px]">
              <SelectValue placeholder="Max tokens" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectLabel>Max tokens</SelectLabel>
                {[500, 1000, 2000, 5000, 10000, 20000, 50000].map((tokens) => (
                  <SelectItem key={tokens} value={tokens.toString()}>
                    {tokens.toLocaleString()}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </Label>
        <Label>
          Temperature
          <Select
            value={currentTemperature.toFixed(1)}
            onValueChange={(temperature) => onSelectTemperature(+temperature)}
          >
            <SelectTrigger className="w-[70px]">
              <SelectValue placeholder="Temperature" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectLabel>Temperature</SelectLabel>
                {[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].map(
                  (temperature) => (
                    <SelectItem
                      key={temperature}
                      value={temperature.toString()}
                    >
                      {temperature.toFixed(1)}
                    </SelectItem>
                  ),
                )}
              </SelectGroup>
            </SelectContent>
          </Select>
        </Label>
      </PopoverContent>
    </Popover>
  );
}
